use gltf::mesh::Reader;
use legion::Entity;
use std::{collections::HashMap, path::Path};

use crate::{
    BoundingBoxComponent, GlobalModelComponent, HierarchyComponent, MeshComponent, TagComponent,
    TransformComponent,
    assets::{
        material_manager::MaterialManager,
        mesh_manager::{self, MeshManager},
        texture_manager::TextureManager,
        vertexdata::MeshVertexData,
    },
    entities::bounding_box::BoundingBox,
    math::*,
    prelude::*,
    renderer::gpu_manager::GPUResourceManager,
};

impl TransformComponent {
    fn from_gltf(g_node: &gltf::Node<'_>) -> Self {
        let (position, r, scale) = g_node.transform().decomposed();
        let quat = Quat::new(r[3], r[0], r[1], r[2]);
        let euler = Euler::from(quat);
        let rotation = [euler.x.0, euler.y.0, euler.z.0];
        Self {
            position,
            rotation,
            scale,
        }
    }
}

pub fn generate_mikktspace_tangents(vertices: &mut [MeshVertexData], indices: &[u32]) {
    use mikktspace::{Geometry, generate_tangents};

    struct Mikkt<'a> {
        vertices: &'a mut [MeshVertexData],
        indices: &'a [u32],
    }

    impl Geometry for Mikkt<'_> {
        fn num_faces(&self) -> usize {
            self.indices.len() / 3
        }

        fn num_vertices_of_face(&self, _: usize) -> usize {
            3
        }

        fn position(&self, face: usize, vert: usize) -> [f32; 3] {
            self.vertices[self.indices[face * 3 + vert] as usize].position
        }

        fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
            self.vertices[self.indices[face * 3 + vert] as usize].normal
        }

        fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
            self.vertices[self.indices[face * 3 + vert] as usize].uv
        }

        fn set_tangent(
            &mut self,
            tangent: [f32; 3],
            _bitangent: [f32; 3],
            _f_mag_s: f32,
            _f_mag_t: f32,
            b_is_orientation_preserving: bool,
            face: usize,
            vert: usize,
        ) {
            let sign = if b_is_orientation_preserving {
                1.0
            } else {
                -1.0
            };
            let idx = self.indices[face * 3 + vert] as usize;

            self.vertices[idx].tangent = [tangent[0], tangent[1], tangent[2], sign];
        }
    }

    let mut geom = Mikkt { vertices, indices };
    generate_tangents(&mut geom);
}

fn extract_indices<'a, F>(reader: &Reader<'a, 'a, F>) -> Result<Vec<u32>, ImportError>
where
    F: Clone + Fn(gltf::Buffer<'a>) -> Option<&'a [u8]>,
{
    let indices = reader
        .read_indices()
        .ok_or(ImportError::MissingIndices)?
        .into_u32()
        .collect::<Vec<u32>>();

    Ok(indices)
}

fn extract_vertices<'a, F>(
    reader: &Reader<'a, 'a, F>,
    indices: &Vec<u32>,
) -> Result<Vec<MeshVertexData>, ImportError>
where
    F: Clone + Fn(gltf::Buffer<'a>) -> Option<&'a [u8]>,
{
    let positions = reader
        .read_positions()
        .ok_or(ImportError::MissingPositions)?;

    let mut vertices: Vec<MeshVertexData> = positions
        .map(|position| MeshVertexData {
            position,
            normal: [0.0, 1.0, 0.0],
            uv: [0.0, 0.0],
            tangent: [0.0, 0.0, 0.0, 0.0],
        })
        .collect();

    if let Some(normals) = reader.read_normals() {
        normals.enumerate().for_each(|(i, normal)| {
            vertices[i].normal = normal;
        });
    } else {
        warn!("Missing Texture Normal");
    }

    if let Some(uvs) = reader.read_tex_coords(0) {
        uvs.into_f32().enumerate().for_each(|(i, uv)| {
            vertices[i].uv = uv;
        });
    } else {
        warn!("Missing UV Coords");
    }

    if let Some(tangent) = reader.read_tangents() {
        tangent.enumerate().for_each(|(i, t)| {
            vertices[i].tangent = t;
        });
    } else {
        generate_mikktspace_tangents(&mut vertices, &indices);
    }

    Ok(vertices)
}

fn extract_bbox(mesh: &gltf::Mesh) -> BoundingBox {
    let mut bounding_box = BoundingBox::new_empty();
    for primitive in mesh.primitives() {
        let b = primitive.bounding_box();
        bounding_box.extend(&b.min);
        bounding_box.extend(&b.max);
    }
    bounding_box
}

fn _get_primitive_mode(mode: gltf::mesh::Mode) -> wgpu::PrimitiveTopology {
    match mode {
        gltf::mesh::Mode::Points => wgpu::PrimitiveTopology::PointList,
        gltf::mesh::Mode::Lines => wgpu::PrimitiveTopology::LineList,
        gltf::mesh::Mode::LineStrip => wgpu::PrimitiveTopology::LineStrip,
        gltf::mesh::Mode::Triangles => wgpu::PrimitiveTopology::TriangleList,
        gltf::mesh::Mode::TriangleStrip => wgpu::PrimitiveTopology::TriangleStrip,
        _ => panic!("Error loading mesh topology isn't supported!"),
    }
}

#[derive(Debug)]
pub enum ImportError {
    Io(std::io::Error),
    Gltf(gltf::Error),
    MissingPositions,
    MissingIndices,
    MeshLoadFailed,
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportError::MissingPositions => write!(f, "Missing POSITION"),
            ImportError::MissingIndices => write!(f, "Missing INDICES"),
            ImportError::MeshLoadFailed => write!(f, "Failed to load mesh"),
            ImportError::Io(e) => write!(f, "IO error: {}", e),
            ImportError::Gltf(e) => write!(f, "glTF error: {}", e),
        }
    }
}

impl From<std::io::Error> for ImportError {
    fn from(e: std::io::Error) -> Self {
        ImportError::Io(e)
    }
}

impl From<gltf::Error> for ImportError {
    fn from(e: gltf::Error) -> Self {
        ImportError::Gltf(e)
    }
}

pub fn load_gltf(
    world: &mut legion::World,
    mesh_manager: &mut MeshManager,
    material_manager: &mut MaterialManager,
    texture_manager: &mut TextureManager,
    gpu_manager: &GPUResourceManager,
    device: &wgpu::Device,
    path: &Path,
) -> Result<Vec<Entity>, ImportError> {
    let timer = std::time::Instant::now();

    if path.extension().unwrap_or_default() != "gltf" {
        warn!("File: {} is not a glTF", path.display());
        return Err(ImportError::MeshLoadFailed);
    }

    let (document, buffers, _) = gltf::import(path)?;

    let images: Vec<gltf::Image<'_>> = document.images().collect();

    let mut material_map = HashMap::new();
    for mat in document.materials() {
        let handle = material_manager.create_material(
            device,
            gpu_manager,
            texture_manager,
            &mat,
            &images,
            path.to_path_buf(),
        );

        material_map.insert(mat.index().unwrap(), handle);
    }

    let mut meshe_handle_map = HashMap::new();
    for g_mesh in document.meshes() {
        for primitive in g_mesh.primitives() {
            let reader = primitive.reader(|b| Some(&buffers[b.index()]));
            let indices = extract_indices(&reader)?;
            let vertices = extract_vertices(&reader, &indices)?;

            let mat_index = primitive.material().index().unwrap_or(0);
            let material = material_map[&mat_index].clone();

            let mesh =
                mesh_manager::create_mesh(&device, &gpu_manager, &vertices, &indices, material);
            let handle = mesh_manager.add_mesh(mesh);
            meshe_handle_map.insert(g_mesh.index(), handle);
        }
    }

    let initial_size = world.len();

    let mut root_entities = Vec::new();
    let mut node_entity_map = HashMap::new();
    println!("Scene len is = {}", document.scenes().len());
    for root_node in document.scenes().next().expect("No scene in Gltf").nodes() {
        let entity = create_entities_recursively(
            world,
            &root_node,
            None,
            &mut meshe_handle_map,
            &mut node_entity_map,
        );
        root_entities.push(entity);
    }

    let num_entities = world.len() - initial_size;
    info!("Create: #{} entities", num_entities);

    debug!("Gltf import is {} ms", timer.elapsed().as_millis());
    debug!("Entity for node 0: {:?}", node_entity_map.get(&0));

    Ok(root_entities)
}

fn create_entities_recursively(
    world: &mut legion::World,
    node: &gltf::Node,
    parent: Option<Entity>,
    meshe_handle_map: &mut HashMap<usize, usize>,
    node_entity_map: &mut HashMap<usize, Entity>,
) -> Entity {
    let name = node.name().map(|s| s.into()).unwrap_or("no-name".into());
    info!("Create node {} id {}", name, node.index());

    let local_transform = TransformComponent::from_gltf(&node);
    let mut parent_transform = Mat4::identity();
    if parent.is_some() {
        let entry = world.entry(parent.unwrap()).unwrap();
        parent_transform = entry.get_component::<GlobalModelComponent>().unwrap().mat;
    }

    let entity = world.push((
        TagComponent { name },
        HierarchyComponent {
            parent,
            ..Default::default()
        },
        GlobalModelComponent {
            mat: parent_transform * local_transform.compute_model_matrix(),
        },
        local_transform,
    ));

    if let Some(g_mesh) = node.mesh() {
        let mut entry = world.entry(entity).unwrap();
        let handle = meshe_handle_map.get(&g_mesh.index()).unwrap().clone();
        let bounding_box = extract_bbox(&g_mesh);
        entry.add_component(MeshComponent { handle });
        entry.add_component(BoundingBoxComponent::new(bounding_box));
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
            meshe_handle_map,
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

    #[test]
    fn should_load_mesh() {
        let (device, queue) = crate::test_utils::get_device_and_queue();

        let gpu_manager = GPUResourceManager::new(&device);
        let mut mesh_manager = MeshManager::new();
        let mut texture_manager = TextureManager::new(device.clone(), queue.clone());
        let mut material_manager = MaterialManager::new(device, &gpu_manager, &mut texture_manager);

        let mut world = legion::World::default();

        let e = load_gltf(
            &mut world,
            &mut mesh_manager,
            &mut material_manager,
            &mut texture_manager,
            &gpu_manager,
            &device,
            std::path::Path::new("./assets/cube/cube.gltf"),
        );

        assert!(e.is_ok());
        assert_eq!(e.ok().iter().len(), 1);
        assert_eq!(Read::<MeshComponent>::query().iter(&world).count(), 1);
        assert_eq!(world.len(), 1)
    }
}
