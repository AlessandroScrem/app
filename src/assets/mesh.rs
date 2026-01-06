use gltf::mesh::Reader;
use legion::{Entity, EntityStore};
use std::{collections::HashMap, path::Path};

use crate::{
    BoundingBoxComponent, GlobalModelComponent, HierarchyComponent, MeshComponent, TagComponent,
    TransformComponent,
    assets::{
        material_manager::{MaterialId, MaterialManager, MaterialPBR},
        mesh_manager::{self, MeshManager, create_gpu_mesh},
        texture_manager::TextureManager,
        vertexdata::MeshVertexData,
    },
    entities::bounding_box::BoundingBox,
    math::*,
    prelude::*,
    renderer::{gpu_manager::GpuManager, gpu_renderer::GpuDevice},
};

pub struct LoadedScene {
    pub meshes: Vec<MeshData>,
    pub materials: Vec<MaterialPBR>,
    pub nodes: Vec<NodeData>,
    pub roots: Vec<usize>, // indici dei nodi root
}

pub struct NodeData {
    pub name: String,
    pub local_transform: TransformComponent,
    pub mesh: Option<usize>,  // index in meshes
    pub children: Vec<usize>, // index in nodes
}

pub struct MeshData {
    pub vertices: Vec<MeshVertexData>,
    pub indices: Vec<u32>,
    pub material: Option<usize>,
    pub bbox: BoundingBox,
}

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
    cmd: &mut legion::systems::CommandBuffer,
    mesh_manager: &mut MeshManager,
    material_manager: &mut MaterialManager,
    texture_manager: &mut TextureManager,
    gpu_manager: &GpuManager,
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

    let mut mesh_handle_map = HashMap::new();
    for g_mesh in document.meshes() {
        for primitive in g_mesh.primitives() {
            let reader = primitive.reader(|b| Some(&buffers[b.index()]));
            let indices = extract_indices(&reader)?;
            let vertices = extract_vertices(&reader, &indices)?;

            let mat_index = primitive.material().index().unwrap_or(0);
            let material_id = material_map[&mat_index].clone();
            let mesh = mesh_manager::create_gpu_mesh(&device, &gpu_manager, &vertices, &indices);
            let handle = mesh_manager.add_mesh(mesh);
            mesh_handle_map.insert(g_mesh.index(), (handle, material_id));
        }
    }

    let mut root_entities = Vec::new();
    let mut node_entity_map = HashMap::new();
    println!("Scene len is = {}", document.scenes().len());
    for root_node in document.scenes().next().expect("No scene in Gltf").nodes() {
        let entity = create_entities_recursively(
            // world,
            cmd,
            &root_node,
            None,
            &mut mesh_handle_map,
            &mut node_entity_map,
        );
        root_entities.push(entity);
    }

    debug!("Gltf import is {} ms", timer.elapsed().as_millis());
    debug!("Entity for node 0: {:?}", node_entity_map.get(&0));

    Ok(root_entities)
}

fn create_entities_recursively(
    cmd: &mut legion::systems::CommandBuffer,
    node: &gltf::Node,
    parent: Option<Entity>,
    mesh_handle_map: &mut HashMap<usize, (usize, MaterialId)>,
    node_entity_map: &mut HashMap<usize, Entity>,
) -> Entity {
    let name = node.name().map(|s| s.into()).unwrap_or("no-name".into());
    info!("Create node {} id {}", name, node.index());

    let local_transform = TransformComponent::from_gltf(&node);

    let entity = cmd.push((
        TagComponent { name },
        HierarchyComponent {
            parent,
            children: Vec::new(),
        },
        GlobalModelComponent::default(),
        local_transform,
    ));

    if let Some(g_mesh) = node.mesh() {
        let (handle, mat_handle) = mesh_handle_map.get(&g_mesh.index()).unwrap().clone();
        let bounding_box = extract_bbox(&g_mesh);

        cmd.add_component(entity, MeshComponent { handle, mat_handle });
        cmd.add_component(entity, BoundingBoxComponent::new(bounding_box));
    }

    // Salva il mapping nodo → entità
    node_entity_map.insert(node.index(), entity);

    // Per ogni figlio: crealo e aggiungilo alla lista children
    let mut children_entities = Vec::new();
    for child in node.children() {
        let child_entity = create_entities_recursively(
            // world,
            cmd,
            &child,
            Some(entity),
            mesh_handle_map,
            node_entity_map,
        );

        cmd.exec_mut(move |w, _| {
            if let Ok(mut entry) = w.entry_mut(entity) {
                if let Ok(h) = entry.get_component_mut::<HierarchyComponent>() {
                    h.children.push(child_entity);
                }
            }
        });
        children_entities.push(child_entity);
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

/*
#[cfg(test)]
mod tests {
    use super::*;
    use legion::query::IntoQuery;
    use legion::world::SubWorld;
    use legion::*;

    #[test]
    fn should_load_mesh() {
        let (device, queue) = crate::test_utils::get_device_and_queue();

        let gpu_manager = GpuManager::new(&device, 32, 32);
        let mut mesh_manager = MeshManager::new();
        let mut texture_manager = TextureManager::new(device.clone(), queue.clone());
        let mut material_manager = MaterialManager::new(device, &gpu_manager, &mut texture_manager);

        let mut world = legion::World::default();
        let subworld = legion::world::SubWorld::from(&world);
        let mut cmd = legion::systems::CommandBuffer::new(&world);

        let e = load_gltf(
            &mut world,
            &mut cmd,
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
} */

// step per load gltf
//     (LoadedScene)
//          |
//          Y
//   create MaterialPBR
//          |
//          Y
//  upload_scene_to_gpu
//          |
//          Y
//   spawn_scene ECS

pub fn load<P: AsRef<Path>>(path: P) -> Result<LoadedScene, ImportError> {
    if path.as_ref().extension().unwrap() != "gltf" {
        error!("File: {} is not a glTF", path.as_ref().display());
        return Err(ImportError::MeshLoadFailed);
    }

    let (gltf, buffers, _) = gltf::import(path.as_ref())?;

    let images: Vec<gltf::Image<'_>> = gltf.images().collect();

    let mut materials = Vec::new();
    for g_mat in gltf.materials() {
        materials.push(create_material(&g_mat, &images, &path));
    }

    let mut meshes = Vec::new();
    for g_mesh in gltf.meshes() {
        for primitive in g_mesh.primitives() {
            let reader = primitive.reader(|b| Some(&buffers[b.index()]));
            let indices = extract_indices(&reader)?;
            let vertices = extract_vertices(&reader, &indices)?;

            let material = primitive.material().index();
            meshes.push(MeshData {
                vertices,
                indices,
                material,
                bbox: extract_bbox(&g_mesh),
            });
        }
    }

    let mut nodes = Vec::new();
    for node in gltf.nodes() {
        let children = node.children().map(|c| c.index()).collect();

        nodes.push(NodeData {
            name: node.name().unwrap_or("no-name").to_string(),
            local_transform: TransformComponent::from_gltf(&node),
            mesh: node.mesh().map(|m| m.index()),
            children,
        });
    }

    let mut has_parent = vec![false; nodes.len()];

    for node in gltf.nodes() {
        for child in node.children() {
            has_parent[child.index()] = true;
        }
    }

    let roots: Vec<usize> = has_parent
        .iter()
        .enumerate()
        .filter(|(_, p)| !**p)
        .map(|(i, _)| i)
        .collect();

    let scene = LoadedScene {
        materials,
        meshes,
        nodes,
        roots,
    };

    Ok(scene)
}

fn create_material<P: AsRef<Path>>(
    gltf_material: &gltf::Material,
    images: &Vec<gltf::Image<'_>>,
    path: P,
) -> MaterialPBR {
    let name = gltf_material.name().unwrap_or("material_no_name");
    let parent_path = path.as_ref().parent().expect("Unable to find parent path");

    fn get_texture_url(
        info: Option<gltf::texture::Info<'_>>,
        path: &Path,
        images: &[gltf::Image<'_>],
    ) -> Option<std::path::PathBuf> {
        let info = info?;
        let image = images.get(info.texture().index())?;

        if let gltf::image::Source::Uri { uri, .. } = image.source() {
            return Path::new(uri).file_name().map(|u| path.join(u));
        }

        None
    }

    // pbr materials
    let pbr = gltf_material.pbr_metallic_roughness();
    let color_factor = pbr.base_color_factor();
    let roughness_factor = pbr.roughness_factor();
    let metallic_factor = pbr.metallic_factor();
    let emissive_factor = gltf_material.emissive_factor();

    let use_color_texture = pbr.base_color_texture().is_some();
    let use_metal_roughness_texture = pbr.base_color_texture().is_some();
    let use_normal_texture = gltf_material.normal_texture().is_some();
    let use_emissive_texture = gltf_material.emissive_texture().is_some();
    let use_occlusion_texture = gltf_material.occlusion_texture().is_some();

    let base_texture_path = get_texture_url(pbr.base_color_texture(), parent_path, &images);
    let met_rough_texture = get_texture_url(pbr.metallic_roughness_texture(), parent_path, &images);
    let emissive_texture_path =
        get_texture_url(gltf_material.emissive_texture(), parent_path, &images);

    let normal_texture_path = gltf_material
        .normal_texture()
        .map(|nt| nt.texture().source().source())
        .and_then(|s| {
            if let gltf::image::Source::Uri { uri, .. } = s {
                Some(parent_path.join(uri))
            } else {
                None
            }
        });

    let normal_scale = gltf_material
        .normal_texture()
        .map(|nt| nt.scale())
        .unwrap_or(1.0);

    let occlusion_texture_path = gltf_material
        .occlusion_texture()
        .map(|ot| ot.texture().source().source())
        .and_then(|s| {
            if let gltf::image::Source::Uri { uri, .. } = s {
                Some(parent_path.join(uri))
            } else {
                None
            }
        });

    let occlusion_strength = gltf_material
        .occlusion_texture()
        .map(|ot| ot.strength())
        .unwrap_or(1.0);

    MaterialPBR {
        name: name.into(),
        base_color_factor: color_factor.into(),
        emissive_factor: Vec3::from(emissive_factor).extend(1.0),
        base_texture_path: base_texture_path.unwrap_or_default(),
        normal_texture_path: normal_texture_path.unwrap_or_default(),
        met_rough_texture_path: met_rough_texture.unwrap_or_default(),
        emissive_texture_path: emissive_texture_path.unwrap_or_default(),
        occlusion_texture_path: occlusion_texture_path.unwrap_or_default(),
        roughness_factor,
        metallic_factor,
        normal_scale,
        occlusion_strength,
        use_color_texture,
        use_metal_roughness_texture,
        use_normal_texture,
        use_emissive_texture,
        use_occlusion_texture,
    }
}

pub struct GpuScene {
    pub mesh_handles: Vec<usize>,
    pub material_handles: Vec<std::path::PathBuf>,
}

pub fn upload_scene_to_gpu(loaded: &LoadedScene, gpu: &mut GpuDevice) -> GpuScene {
    let material_handles = loaded
        .materials
        .iter()
        .map(|m| gpu.mat_mgr.create(gpu.device, gpu.gpu_mgr, gpu.texure_mgr, m))
        .collect();

    let mesh_handles = loaded
        .meshes
        .iter()
        .map(|m| {
            let gpu_mesh = create_gpu_mesh(gpu.device, gpu.gpu_mgr, &m.vertices, &m.indices);
            gpu.mesh_mgr.add_mesh(gpu_mesh)
        })
        .collect();

    GpuScene {
        mesh_handles,
        material_handles,
    }
}

pub fn spawn_scene(world: &mut legion::World, loaded: &LoadedScene, gpu: &GpuScene) {
    let mut node_to_entity = Vec::with_capacity(loaded.nodes.len());

    // 1️⃣ crea tutte le entity
    for node in &loaded.nodes {
        let name = node.name.clone();
        let entity = world.push((
            TagComponent { name },
            TransformComponent::from(node.local_transform.clone()),
            HierarchyComponent::default(),
            GlobalModelComponent::default(),
        ));
        node_to_entity.push(entity);
    }

    // 2️⃣ assegna mesh + material
    for (i, node) in loaded.nodes.iter().enumerate() {
        if let Some(mesh_idx) = node.mesh {
            let entity = node_to_entity[i];
            let mesh = &loaded.meshes[mesh_idx];
            let mut entry = world.entry(entity).unwrap();

            // MeshComponent
            entry.add_component(MeshComponent {
                handle: gpu.mesh_handles[mesh_idx],
                mat_handle: mesh
                    .material
                    .map(|m| gpu.material_handles[m].clone())
                    .unwrap(),
            });

            // BoundingBoxComponent
            let bbox = &mesh.bbox;
            entry.add_component(BoundingBoxComponent {
                bounding_box: bbox.clone(),
                global_bounding_box: bbox.clone(),
            });
        }
    }

    // 3️⃣ collega la gerarchia
    for (i, node) in loaded.nodes.iter().enumerate() {
        let parent = node_to_entity[i];

        for &child_idx in &node.children {
            let child = node_to_entity[child_idx];

            world
                .entry_mut(parent)
                .unwrap()
                .get_component_mut::<HierarchyComponent>()
                .unwrap()
                .children
                .push(child);

            world
                .entry_mut(child)
                .unwrap()
                .get_component_mut::<HierarchyComponent>()
                .unwrap()
                .parent = Some(parent);
        }
    }
}
