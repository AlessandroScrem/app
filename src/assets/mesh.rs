use legion::Entity;
use std::{collections::HashMap, path::Path, time::Instant};
use wgpu::util::DeviceExt;

use crate::{
    BoundingBoxComponent, GlobalModelComponent, HierarchyComponent, MeshComponent, TagComponent,
    TransformComponent,
    assets::{
        material_manager::{Material, MaterialManager},
        texture_manager::TextureManager,
        vertexdata::MeshVertexData,
    },
    entities::bounding_box::BoundingBox,
    math::*,
    prelude::*,
    renderer::gpu_manager::{GPUResourceManager, LayoutKind},
};

fn quat_to_euler_rad_array(q: Quat) -> [f32; 3] {
    let euler: Euler<Rad<f32>> = q.into();
    [euler.x.0, euler.y.0, euler.z.0]
}

pub struct Node {
    pub index: usize, // glTF index
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub mesh_id: Option<usize>,
    pub local_transform: Mat4,
    pub global_transform: Mat4,
    pub name: Option<String>,
}

impl Node {
    pub fn from_gltf(g_node: &gltf::Node<'_>) -> Node {
        let mesh_id = g_node.mesh().map(|m| m.index());
        let children: Vec<_> = g_node.children().map(|g_node| g_node.index()).collect();
        let name = g_node.name().map(|s| s.into());
        let local_transform = g_node.transform().matrix().into();

        Node {
            index: g_node.index(),
            parent: None,
            children,
            mesh_id,
            local_transform,
            global_transform: Mat4::identity(),
            name,
        }
    }
}

pub struct SubMesh {
    pub vertices: Vec<MeshVertexData>,
    pub indices: Vec<u32>,
    pub vmin: [f32; 3],
    pub vmax: [f32; 3],
    pub(crate) vertex_buffer: Option<wgpu::Buffer>,
    pub(crate) index_buffer: Option<wgpu::Buffer>,
    pub(crate) index_count: usize,
    pub primitive_topology: wgpu::PrimitiveTopology,
    pub material: Material,
}

impl SubMesh {
    fn from_primitive(
        primitive: &gltf::Primitive,
        buffers: Vec<gltf::buffer::Data>,
        images: &Vec<gltf::Image>,
        path: &Path,
        material_manager: &mut MaterialManager,
        texture_manager: &mut TextureManager,
        device: &wgpu::Device,
    ) -> Self {
        let timer = std::time::Instant::now();

        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

        let positions = reader
            .read_positions()
            .expect("primitives must have the POSITION attribute ");
        let indices = reader
            .read_indices()
            .expect("primitives must have the INDICES attribute ")
            .into_u32()
            .collect::<Vec<u32>>();

        let mut bbox = BoundingBox::new_empty();
        let mut vertices: Vec<MeshVertexData> = positions
            .map(|position| {
                //extend bbox
                bbox.extend(&position);
                MeshVertexData {
                    position,
                    normal: [0.0, 1.0, 0.0],
                    color: [0.5, 0.5, 0.5],
                    uv: [0.0, 0.0],
                }
            })
            .collect();

        if let Some(normals) = reader.read_normals() {
            normals.enumerate().for_each(|(i, normal)| {
                vertices[i].normal = normal;
            });
        }

        if let Some(uvs) = reader.read_tex_coords(0) {
            uvs.into_f32().enumerate().for_each(|(i, uv)| {
                vertices[i].uv = uv;
            });
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Index Buffer"),
            contents: &bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let index_count = indices.len();

        // begin material
        let gltf_material: gltf::Material<'_> = primitive.material();
        let material = material_manager.create_material(
            texture_manager,
            &gltf_material,
            &images,
            path.to_path_buf(),
        );

        info!(
            "--\t create material: {} is {} ms",
            primitive.material().name().unwrap_or("no_name"),
            timer.elapsed().as_millis()
        );

        let primitive_topology = Self::get_primitive_mode(primitive.mode());

        SubMesh {
            vertices,
            indices,
            vertex_buffer: Some(vertex_buffer),
            index_buffer: Some(index_buffer),
            index_count,
            primitive_topology,
            material,
            vmin: bbox.min,
            vmax: bbox.max,
        }
    }

    fn get_primitive_mode(mode: gltf::mesh::Mode) -> wgpu::PrimitiveTopology {
        match mode {
            gltf::mesh::Mode::Points => wgpu::PrimitiveTopology::PointList,
            gltf::mesh::Mode::Lines => wgpu::PrimitiveTopology::LineList,
            gltf::mesh::Mode::LineStrip => wgpu::PrimitiveTopology::LineStrip,
            gltf::mesh::Mode::Triangles => wgpu::PrimitiveTopology::TriangleList,
            gltf::mesh::Mode::TriangleStrip => wgpu::PrimitiveTopology::TriangleStrip,
            _ => panic!("Error loading mesh topology isn't supported!"),
        }
    }
}

pub struct Mesh {
    pub name: String,
    pub submeshes: Vec<SubMesh>,
    pub model_uniform_buffer: wgpu::Buffer,
    pub model_bind_group: wgpu::BindGroup,
    pub vmin: [f32; 3],
    pub vmax: [f32; 3],
}

impl Mesh {
    fn from_gltf(
        gltf_mesh: &gltf::Mesh,
        buffers: Vec<gltf::buffer::Data>,
        images: &Vec<gltf::Image>,
        path: &Path,
        material_manager: &mut MaterialManager,
        texture_manager: &mut TextureManager,
        gpu_resource_manager: &GPUResourceManager,
        device: &wgpu::Device,
    ) -> Self {
        let timer = Instant::now();

        let name = gltf_mesh.name().unwrap_or("mesh").to_string();
        let submeshes: Vec<SubMesh> = gltf_mesh
            .primitives()
            .map(|prim| {
                SubMesh::from_primitive(
                    &prim,
                    buffers.clone(),
                    &images,
                    path,
                    material_manager,
                    texture_manager,
                    device,
                )
            })
            .collect();

        let mesh_bbox = submeshes
            .iter()
            .fold(BoundingBox::new_empty(), |mut acc, submesh| {
                acc.extend(&submesh.vmin);
                acc.extend(&submesh.vmax);
                acc
            });

        let model_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Model Uniform Buffer"),
            contents: bytemuck::cast_slice(&[crate::renderer::uniform::ModelUniform::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let model_bind_group_layout = gpu_resource_manager.get_layout(LayoutKind::Model);

        let model_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &model_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: model_uniform_buffer.as_entire_binding(),
            }],
            label: Some("Model Bind Group"),
        });

        info!(
            "Loading mesh {} took {} ms",
            path.display(),
            timer.elapsed().as_millis()
        );

        Mesh {
            name,
            submeshes,
            model_uniform_buffer,
            model_bind_group,
            vmin: mesh_bbox.min,
            vmax: mesh_bbox.max,
        }
    }
}

pub fn load_gltf(
    world: &mut legion::World,
    material_manager: &mut MaterialManager,
    texture_manager: &mut TextureManager,
    gpu_resource_manager: &GPUResourceManager,
    device: &wgpu::Device,
    path: &Path,
) ->Option<Entity>{
    let timer = std::time::Instant::now();

    if path.extension().unwrap_or_default() != "gltf" {
        warn!("File: {} is not a glTF", path.display());
        return None;
    }

    let (document, buffers, _) = match gltf::import(path) {
        Ok((doc, buffers, images)) => (doc, buffers, images),
        Err(e) => {
            warn!("Error loading Gltf {e}");
            return None;
        }
    };

    let images: Vec<gltf::Image<'_>> = document.images().collect();

    let mut indexed_meshes: HashMap<usize, (Mesh, BoundingBoxComponent)> = document
        .meshes()
        .map(|gltf_mesh| {
            let mesh = Mesh::from_gltf(
                &gltf_mesh,
                buffers.clone(),
                &images,
                path,
                material_manager,
                texture_manager,
                gpu_resource_manager,
                device,
            );
            let bounding_box = BoundingBoxComponent::new(device, (mesh.vmin, mesh.vmax).into());
            (gltf_mesh.index(), (mesh, bounding_box))
        })
        .collect();

    debug!("Numero mesh caricate {}", indexed_meshes.len());

    let scene = document
        .default_scene()
        .unwrap_or_else(|| document.scenes().next().expect("No scene in glTF file"));
    let mut root_entities = Vec::new();
    let mut node_entity_map = HashMap::new();

    let initial_size = world.len();

    for root_node in scene.nodes() {
        let entity = create_entities_recursively(
            world,
            &root_node,
            None,
            &mut indexed_meshes,
            &mut node_entity_map,
        );
        root_entities.push(entity);
    }

    let num_entities = world.len() - initial_size;
    info!("Create: #{} entities", num_entities);

    if let Some(e) = node_entity_map.get(&0) {
        debug!("Entity for node 0: {:?}", e);
    }
    
    info!("Gltf import is {} ms", timer.elapsed().as_millis());
    info!("Root entities: {:?}", root_entities);
    

    if let Some(root) = root_entities.first() {
        info!("First root entity: {:?}", root);
        Some(root.clone())
    }
     else {
        None
    }
}

fn create_entities_recursively(
    world: &mut legion::World,
    node: &gltf::Node,
    parent: Option<Entity>,
    indexed_meshes: &mut HashMap<usize, (Mesh, BoundingBoxComponent)>,
    node_entity_map: &mut HashMap<usize, Entity>,
) -> Entity {
    // Crea l'entità per questo nodo
    let name = node.name().map(|s| s.into()).unwrap_or("no-name".into());
    info!("Create node {} id {}", name, node.index());

    let (position, r, scale) = node.transform().decomposed();
    let rotation = quat_to_euler_rad_array(Quat::new(r[3], r[0], r[1], r[2]));

    let transform = TransformComponent {
        position,
        rotation,
        scale,
    };
    let hierarchy = HierarchyComponent {
        parent,
        children: Vec::new(),
    };
    let parent_transform = match parent {
        Some(parent) => {
            let entry = world.entry(parent).unwrap();
            entry
                .get_component::<GlobalModelComponent>()
                .unwrap_or(&GlobalModelComponent::default())
                .mat
        }
        None => GlobalModelComponent::default().mat,
    };
    let global_model =
        GlobalModelComponent::from(parent_transform * transform.compute_model_matrix());

    let entity = world.push((
        TagComponent { name },
        transform.clone(),
        hierarchy,
        global_model,
    ));

    if let Some(g_mesh) = node.mesh() {
        let mut entry = world.entry(entity).unwrap();
        let (mesh, bbox_component) = indexed_meshes.remove(&g_mesh.index()).unwrap();
        entry.add_component(MeshComponent { data: mesh });
        entry.add_component(bbox_component);
    }

    // Salva il mapping nodo → entità
    node_entity_map.insert(node.index(), entity);

    // Per ogni figlio: crealo e aggiungilo alla lista children
    let mut children_entities = Vec::new();
    for child in node.children() {
        let child_entity = create_entities_recursively(
            world,
            &child,
            Some(entity),
            indexed_meshes,
            node_entity_map,
        );
        children_entities.push(child_entity);
    }

    // Aggiorna la lista dei figli
    if !children_entities.is_empty() {
        let mut entry = world.entry(entity).unwrap();
        entry
            .get_component_mut::<HierarchyComponent>()
            .unwrap()
            .children = children_entities;
    }

    entity
}

#[allow(dead_code)]
fn print_gltf_document(document: &gltf::Document) {
    fn print_mesh(mesh: gltf::Mesh) {
        println!(
            " - Mesh #{}: {}",
            mesh.index(),
            mesh.name().unwrap_or("<Unnamed>")
        );
        for primitive in mesh.primitives() {
            let index = primitive.indices().unwrap();
            println!(
                " - Primitive #{} Index Count {}",
                primitive.index(),
                index.count()
            );
        }
    }

    fn print_node(node: &gltf::Node) {
        println!(
            " - Node {} ({}) - transform: TRS {:?}",
            node.index(),
            node.name().unwrap_or("<Unnamed>"),
            node.transform().decomposed()
        );
        if let Some(mesh) = node.mesh() {
            print_mesh(mesh);
        }

        for child in node.children() {
            println!(" - Child Node {}", child.index());
        }
        println!();
    }

    document.nodes().for_each(|node| {
        print_node(&node);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use legion::query::IntoQuery;
    use legion::*;
    use std::sync::Arc;

    #[test]
    fn should_load_mesh() {
        let (device, queue) = crate::test_utils::get_device_and_queue();

        let gpu_manager = GPUResourceManager::new(&device);
        let gpu_manager = Arc::new(gpu_manager);
        let mut material_manager = MaterialManager::new(device.clone(), gpu_manager.clone());

        let mut texture_manager = TextureManager::new(device.clone(), queue.clone());

        let mut world = legion::World::default();

        load_gltf(
            &mut world,
            &mut material_manager,
            &mut texture_manager,
            &gpu_manager,
            &device,
            std::path::Path::new("./assets/cube/cube.gltf"),
        );

        assert_eq!(Read::<MeshComponent>::query().iter(&world).count(), 1);
        assert_eq!(world.len(), 1)
    }
}
