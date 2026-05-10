use super::*;

use crate::entities::EntityRawU64;
use crate::input::Input;
use crate::picking::PickObject;
use crate::prelude::math::*;
use crate::uniform::LightsUniform;
use crate::{entities::bounding_box_impl::BBoxVertexData, uniform::LightUniform};
use legion::{Entity, World};

pub struct MeshDraw {
    pub mesh: MeshId,
    pub submesh_index_range: std::ops::Range<u32>,
    pub material: MaterialId,
    pub entity_id: Entity,
    pub transform: Mat4,
    pub instance_index: u32,
}

pub struct PickingData {
    pub mouse_pos_x: u32,
    pub mouse_pos_y: u32,
}

pub struct BBoxData {
    pub vertexbuffer: wgpu::Buffer,
    pub count: u32,
}

pub struct FrameData {
    // geometry
    pub opaque: Vec<MeshDraw>,
    pub transmission: Vec<MeshDraw>,
    pub transparent: Vec<MeshDraw>,
    pub bbox_bufferdata: Option<BBoxData>,
    pub lights: Option<LightsUniform>,
    pub instances: Vec<vertexdata::VertexInstace>,

    // flags / tasks
    pub axis_enable: bool,
    pub outline_selected: bool,
    pub picking: Option<PickingData>,
    pub skybox_enable: Option<bool>,
    pub build_mips: Option<bool>,
}

pub struct FrameBuilder {}
impl FrameBuilder {
    pub fn build(
        world: &World,
        device: &wgpu::Device,
        asset: &AssetManager,
        selected: Option<Entity>,
        pickobject: &PickObject,
        input: &Input,
        globals: &Globals,
    ) -> FrameData {
        let mut frame = FrameData {
            opaque: Vec::new(),
            transmission: Vec::new(),
            transparent: Vec::new(),
            bbox_bufferdata: None,
            lights: None,
            outline_selected: false,
            picking: None,
            skybox_enable: None,
            build_mips: None,
            axis_enable: false,
            instances: Vec::new(),
        };
        Self::build_geometry(world, asset, &mut frame);
        Self::build_instances(&mut frame);
        Self::build_picking(input, pickobject, &mut frame);
        Self::build_bbox_data(device, world, globals, &mut frame);
        Self::build_light_data(world, globals, &mut frame);
        frame.build_mips = (!frame.transmission.is_empty()).then(|| globals.mips_cs);
        frame.outline_selected = selected.is_some();
        frame.skybox_enable = globals.skybox_enable.then(|| globals.skybox_enable_blur);
        frame.axis_enable = globals.axis_enable;

        debug!(
            "Opaque Count: {}, Transmission Count: {}, Total: {}",
            frame.opaque.len(),
            frame.transmission.len(),
            frame.opaque.len() + frame.transmission.len() + frame.transparent.len()
        );
        frame
    }

    fn build_geometry(world: &World, asset: &AssetManager, frame: &mut FrameData) {
        use legion::IntoQuery;
        let mut query = <(Entity, &MeshComponent, &GlobalModelComponent)>::query();
        for (entity, mesh_comp, global_mat) in query.iter(world) {
            if let Some(mesh) = asset.meshes.get(mesh_comp.handle) {
                for submesh in mesh.submeshes.iter() {
                    if let Some(material) = asset.materials.get_desc(submesh.material) {
                        debug_assert!(
                            global_mat.mat.determinant() > 0.0,
                            "matrix determinant is negative"
                        );
                        let item = MeshDraw {
                            mesh: mesh_comp.handle,
                            submesh_index_range: submesh.index_range.clone(),
                            entity_id: *entity,
                            material: submesh.material,
                            transform: global_mat.mat,
                            instance_index: 0, // TODO!
                        };
                        Self::classify(item, material, frame);
                    }
                }
            }
        }
    }

    fn build_instances(frame: &mut FrameData) {
        use vertexdata::VertexInstace;
        for item in frame.opaque.iter_mut() {
            item.instance_index = frame.instances.len() as u32;

            let model = VertexInstace::new(item.transform, item.entity_id.as_raw_u64());
            frame.instances.push(model);
        }
        
        for item in frame.transmission.iter_mut() {
            item.instance_index = frame.instances.len() as u32;
            
            let model = VertexInstace::new(item.transform, item.entity_id.as_raw_u64());
            frame.instances.push(model);
        }
    }

    fn build_picking(input: &Input, pickobject: &PickObject, frame: &mut FrameData) {
        if input.is_cursor_moved() && !pickobject.pending {
            let pick_data = PickingData {
                mouse_pos_x: input.mouse_position.x as u32,
                mouse_pos_y: input.mouse_position.y as u32,
            };
            frame.picking = Some(pick_data);
        }
    }

    fn classify(item: MeshDraw, mat: &MaterialDesc, frame: &mut FrameData) {
        if mat.is_transmissive() {
            frame.transmission.push(item);
        } else if mat.is_transparent() {
            frame.transparent.push(item);
        } else {
            frame.opaque.push(item);
        }
    }

    fn build_bbox_data(
        device: &wgpu::Device,
        world: &World,
        globals: &Globals,
        frame: &mut FrameData,
    ) {
        if !globals.bbox_enable {
            return;
        }

        fn create_buffer(device: &wgpu::Device, vertices: Vec<BBoxVertexData>) -> BBoxData {
            use entities::bounding_box_impl::VERTICES;
            use wgpu::util::DeviceExt;
            let vertexbuffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("BBox Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let count = (vertices.len() * VERTICES) as u32;

            BBoxData {
                count,
                vertexbuffer,
            }
        }

        let axis_aligned = globals.bbox_axis_aligned;

        // -------- BoundingBox --------
        use legion::IntoQuery;
        let mut bbox_query = <(&BoundingBoxComponent, &GlobalModelComponent)>::query();

        let vertexdata = bbox_query
            .iter(world)
            .map(|(bbox, global_model)| {
                if axis_aligned {
                    bbox.gen_aabb_vertices()
                } else {
                    bbox.gen_obb_vertices(&global_model.mat)
                }
            })
            .collect::<Vec<_>>();

        if !vertexdata.is_empty() {
            let bbox_buffer = create_buffer(device, vertexdata);
            frame.bbox_bufferdata = Some(bbox_buffer);
        }
    }

    fn build_light_data(world: &World, globals: &Globals, frame: &mut FrameData) {
        // -------- Lights --------
        let mut light_uniform = LightsUniform::default();
        light_uniform.enabled = globals.light_enable.into();

        use legion::IntoQuery;
        let mut light_query = <(Entity, &LightComponent)>::query();
        for (i, (entity, light)) in light_query
            .iter(world)
            .take(uniform::MAX_LIGHTS)
            .enumerate()
        {
            let data = LightUniform {
                entity_id: entities::EntityRawU64::as_raw_u64(entity),
                ..light.into()
            };
            light_uniform.count = (i + 1) as u32;
            light_uniform.lights[i] = data;
        }

        frame.lights = Some(light_uniform);
    }
}
