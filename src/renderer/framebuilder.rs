use std::collections::HashMap;

use super::*;

use crate::EntityRawU64;
use crate::assets::asset_manager::AssetManager;
use crate::assets::{LinesVertexData, MaterialId, MeshId, VertexInstance};
use crate::ecs::components::*;
use crate::globals::Globals;
use crate::math::*;
use crate::prelude::trace;
use crate::renderer::uniform::{LightUniform, LightsUniform};

use legion::{Entity, World};

pub struct InstanceBatch {
    pub mesh: MeshId,
    pub material: MaterialId,

    pub submesh_index_range: std::ops::Range<u32>,

    pub instance_start: u32,
    pub instance_count: u32,
}

#[derive(Hash, PartialEq, Eq)]
struct BatchKey {
    mesh: MeshId,
    material: MaterialId,

    index_start: u32,
    index_end: u32,
}

pub struct PickingData {
    pub mouse_pos_x: u32,
    pub mouse_pos_y: u32,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct DrawStats {
    pub draw_calls: u32,
    pub instances: u32,
}

fn compute_stats(batches: &[InstanceBatch]) -> DrawStats {
    let draw_calls = batches.len() as u32;

    let instances = batches.iter().map(|b| b.instance_count).sum::<u32>();
    DrawStats {
        draw_calls,
        instances,
    }
}

pub trait LineSink {
    fn line(&mut self, a: Vec3, b: Vec3, color: Vec3);
    fn arrow(&mut self, a: Vec3, b: Vec3, color: Vec3);
}

pub trait LineDrawable {
    fn emit(&self, sink: &mut dyn LineSink);
}

impl LineSink for LineBuilder {
    fn line(&mut self, a: Vec3, b: Vec3, color: Vec3) {
        self.push(LinesVertexData {
            position: a.into(),
            color: color.into(),
        });

        self.push(LinesVertexData {
            position: b.into(),
            color: color.into(),
        });
    }

    fn arrow(&mut self, a: Vec3, b: Vec3, color: Vec3) {
        // linea principale
        self.line(a, b, color);

        let dir = (b - a).normalize();
        let length = (b - a).magnitude();

        let head_len = length * 0.2;
        let head_width = head_len * 0.5;

        let tip = b;
        let base = b - dir * head_len;

        let mut side = dir.cross(Vec3::unit_y()).normalize();
        if side.magnitude2() < 0.001 {
            side = dir.cross(Vec3::unit_x()).normalize();
        }

        let left = base - side * head_width;
        let right = base + side * head_width;

        self.line(left, tip, color);
        self.line(right, tip, color);
    }
}

pub struct ObjectOrientedBoundingBox<'a> {
    pub bbox: &'a crate::BoundingBox,
    pub transform: &'a Mat4,
}
pub struct AxisAlignedBoundingBox<'a> {
    pub bbox: &'a crate::BoundingBox,
}

fn emit_box_edges(corners: &[Vec3; 8], color: Vec3, sink: &mut dyn LineSink) {
    #[rustfmt::skip]
    const BOX_EDGES: [(usize, usize); 12] = [
        (0, 1), (1, 2), (2, 3), (3, 0), 
        (4, 5), (5, 6), (6, 7), (7, 4), 
        (0, 4), (1, 5), (2, 6), (3, 7), 
    ];

    for (a, b) in BOX_EDGES {
        sink.line(corners[a], corners[b], color);
    }
}

impl<'a> LineDrawable for ObjectOrientedBoundingBox<'a> {
    fn emit(&self, sink: &mut dyn LineSink) {
        let color = crate::colors::CYAN_COLOR.into();
        let corners = self.bbox.gen_corners();

        let corners = corners.map(|c| (self.transform * c.extend(1.0)).truncate());

        emit_box_edges(&corners, color, sink);
    }
}

impl<'a> LineDrawable for AxisAlignedBoundingBox<'a> {
    fn emit(&self, sink: &mut dyn LineSink) {
        let color: Vec3 = crate::colors::CYAN_COLOR.into();
        let corners = self.bbox.gen_corners();

        emit_box_edges(&corners, color, sink);
    }
}

impl LineDrawable for LightComponent {
    fn emit(&self, sink: &mut dyn LineSink) {
        use crate::colors;

        let mut corners = vec![
            Vec3::new(-1.0, -1.0, 0.0), // Near-bottom-left
            Vec3::new(1.0, -1.0, 0.0),  // Near-bottom-right
            Vec3::new(1.0, 1.0, 0.0),   // Near-top-right
            Vec3::new(-1.0, 1.0, 0.0),  // Near-top-left
            Vec3::new(-1.0, -1.0, 1.0), // Far-bottom-left
            Vec3::new(1.0, -1.0, 1.0),  // Far-bottom-right
            Vec3::new(1.0, 1.0, 1.0),   // Far-top-right
            Vec3::new(-1.0, 1.0, 1.0),  // Far-top-left
        ];

        let mat = self.get_view_proj_matrix();
        let inverse_light_space_matrix = mat.invert().unwrap_or(Mat4::identity());
        for vertex in corners.iter_mut() {
            let v = inverse_light_space_matrix * vertex.extend(1.0);
            *vertex = v.truncate();
        }

        let near = [corners[0], corners[1], corners[2], corners[3]];
        let far = [corners[4], corners[5], corners[6], corners[7]];

        // Near clip
        sink.line(near[0], near[1], colors::RED_COLOR.into());
        sink.line(near[1], near[2], colors::RED_COLOR.into());
        sink.line(near[2], near[3], colors::RED_COLOR.into());
        sink.line(near[3], near[0], colors::RED_COLOR.into());
        // Far clip
        sink.line(far[0], far[1], colors::BLUE_COLOR.into());
        sink.line(far[1], far[2], colors::BLUE_COLOR.into());
        sink.line(far[2], far[3], colors::BLUE_COLOR.into());
        sink.line(far[3], far[0], colors::BLUE_COLOR.into());
        // Linees connecting near
        sink.line(near[0], far[0], colors::GREEN_COLOR.into());
        sink.line(near[1], far[1], colors::GREEN_COLOR.into());
        sink.line(near[2], far[2], colors::GREEN_COLOR.into());
        sink.line(near[3], far[3], colors::GREEN_COLOR.into());

        let origin = Vec3::new(0.0, 0.0, 0.0);
        let position: Vec3 = self.get_position().into();
        let direction = (origin - position).normalize();
        let target = position + direction * 20.0;

        sink.arrow(position, target, colors::GREEN_COLOR.into());
    }
}

type LineBuilder = Vec<LinesVertexData>;

pub struct FrameData {
    // geometry
    pub opaque_batches: Vec<InstanceBatch>,
    pub transmission_batches: Vec<InstanceBatch>,
    pub lines: Vec<LinesVertexData>,
    pub instances: Vec<VertexInstance>,

    // runtime data
    pub lights: Option<LightsUniform>,

    // flags / tasks
    pub axis_enable: bool,
    pub entity_selected: Option<Entity>,
    pub picking: Option<PickingData>,
    pub skybox_enable: Option<bool>,
    pub build_mips: Option<bool>,

    // stats
    pub opaque_stats: DrawStats,
    pub transmission_stats: DrawStats,
}

pub struct FrameBuilder {}
impl FrameBuilder {
    pub fn build(
        world: &World,
        asset: &AssetManager,
        selected: Option<Entity>,
        picking_data: Option<PickingData>,
        globals: &Globals,
    ) -> FrameData {
        let mut frame = FrameData {
            opaque_batches: Vec::new(),
            transmission_batches: Vec::new(),
            lines: Vec::new(),
            lights: None,
            entity_selected: None,
            picking: picking_data,
            skybox_enable: None,
            build_mips: None,
            axis_enable: false,
            instances: Vec::new(),
            opaque_stats: DrawStats::default(),
            transmission_stats: DrawStats::default(),
        };
        Self::build_geometry(world, asset, &mut frame);
        Self::build_bbox_data(world, globals, &mut frame);
        Self::build_light_data(world, globals, &mut frame);
        Self::build_light_frustum(world, globals, &mut frame);
        frame.build_mips = (!frame.transmission_batches.is_empty()).then(|| globals.mips_cs);
        frame.entity_selected = selected;
        frame.skybox_enable = globals.skybox_enable.then(|| globals.skybox_enable_blur);
        frame.axis_enable = globals.axis_enable;

        frame.opaque_stats = compute_stats(&frame.opaque_batches);
        frame.transmission_stats = compute_stats(&frame.transmission_batches);

        trace!(
            "Opaque Stats: {:?}, Transmission Stats: {:?}, Total DrawCall: {}",
            frame.opaque_stats,
            frame.transmission_stats,
            frame.opaque_stats.draw_calls + frame.transmission_stats.draw_calls
        );
        frame
    }

    fn build_geometry(world: &World, asset: &AssetManager, frame: &mut FrameData) {
        use legion::IntoQuery;
        let mut opaque_map: HashMap<BatchKey, Vec<VertexInstance>> = HashMap::new();
        let mut transmission_map: HashMap<BatchKey, Vec<VertexInstance>> = HashMap::new();
        use crate::assets::MaterialAsset;
        use crate::assets::MeshAsset;

        let mut query = <(Entity, &MeshComponent, &GlobalModelComponent)>::query();
        for (entity, mesh_comp, global_mat) in query.iter(world) {
            let Some(mesh) = asset.get::<MeshAsset>(mesh_comp.handle) else {
                continue;
            };

            for submesh in mesh.desc.submeshes.iter() {
                let Some(material) = asset.get::<MaterialAsset>(submesh.material) else {
                    continue;
                };
                debug_assert!(
                    global_mat.mat.determinant() > 0.0,
                    "matrix determinant is negative"
                );

                let key = BatchKey {
                    mesh: mesh_comp.handle,
                    material: submesh.material,

                    index_start: submesh.index_range.start,
                    index_end: submesh.index_range.end,
                };

                // -------------------------------------------------
                // BUILD INSTANCE
                // -------------------------------------------------

                let model = global_mat.mat;
                let instance = VertexInstance::new(model, entity.as_raw_u64());

                // -------------------------------------------------
                // CLASSIFY
                // -------------------------------------------------

                if material.desc.is_transmissive() {
                    transmission_map.entry(key).or_default().push(instance);
                } else if !material.desc.is_transparent() {
                    opaque_map.entry(key).or_default().push(instance);
                }
            }
        }

        // ---------------------------------------------------------
        // BUILD FINAL BATCHES
        // ---------------------------------------------------------
        fn flush_batches(
            map: HashMap<BatchKey, Vec<VertexInstance>>,
            batches: &mut Vec<InstanceBatch>,
            instances: &mut Vec<VertexInstance>,
        ) {
            for (key, batch_instances) in map {
                let start = instances.len() as u32;
                let count = batch_instances.len() as u32;

                instances.extend(batch_instances);

                batches.push(InstanceBatch {
                    mesh: key.mesh,
                    material: key.material,

                    submesh_index_range: key.index_start..key.index_end,

                    instance_start: start,
                    instance_count: count,
                });
            }
        }

        flush_batches(opaque_map, &mut frame.opaque_batches, &mut frame.instances);
        flush_batches(
            transmission_map,
            &mut frame.transmission_batches,
            &mut frame.instances,
        );
    }

    fn build_bbox_data(world: &World, globals: &Globals, frame: &mut FrameData) {
        if !globals.bbox_enable {
            return;
        }

        let axis_aligned = globals.bbox_axis_aligned;

        // -------- BoundingBox --------
        use legion::IntoQuery;
        let mut bbox_query = <(&BoundingBoxComponent, &GlobalModelComponent)>::query();

        bbox_query.for_each(world, |(b, gm)| {
            if axis_aligned {
                AxisAlignedBoundingBox {
                    bbox: &b.global_bounding_box,
                }
                .emit(&mut frame.lines);
            } else {
                ObjectOrientedBoundingBox {
                    bbox: &b.bounding_box,
                    transform: &gm.mat,
                }
                .emit(&mut frame.lines);
            }
        });
    }

    fn build_light_frustum(world: &World, _globals: &Globals, frame: &mut FrameData) {
        use legion::IntoQuery;
        let mut light_query = <&LightComponent>::query();
        for light in light_query
            .iter(world)
            .filter(|l| l.frustum)
            .take(uniform::MAX_LIGHTS)
        {
            light.emit(&mut frame.lines);
        }
    }

    fn build_light_data(world: &World, globals: &Globals, frame: &mut FrameData) {
        // -------- Lights --------
        let mut lights_uniform = LightsUniform::default();
        lights_uniform.enabled = globals.light_enable.into();

        use legion::IntoQuery;
        let mut light_query = <(Entity, &LightComponent)>::query();
        for (i, (entity, light)) in light_query
            .iter(world)
            .filter(|(_, l)| l.enabled)
            .take(uniform::MAX_LIGHTS)
            .enumerate()
        {
            let data = LightUniform {
                entity_id: EntityRawU64::as_raw_u64(entity),
                ..light.into()
            };
            lights_uniform.count = (i + 1) as u32;
            lights_uniform.lights[i] = data;
        }

        frame.lights = Some(lights_uniform);
    }
}
