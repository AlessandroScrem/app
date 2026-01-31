use gltf::mesh::{Mode, Reader};
use legion::EntityStore;
use std::path::Path;

use crate::{
    BoundingBoxComponent, GlobalModelComponent, HierarchyComponent, TagComponent,
    TransformComponent,
    assets::{
        MaterialDesc, MaterialId, MaterialKey, MeshDesc, MeshId, MeshKey, SubMesh, TextureKey,
        asset_manager::AssetManager, material_manager::MaterialPBR, vertexdata::MeshVertexData,
    },
    math::*,
    prelude::*,
};

pub struct LoadedScene {
    pub meshes: Vec<MeshId>,
    pub materials: Vec<MaterialId>,
    pub nodes: Vec<NodeData>,
    pub roots: Vec<usize>, // indici dei nodi root
}

pub struct NodeData {
    pub name: String,
    pub local_transform: TransformComponent,
    pub mesh: Option<usize>,  // index in meshes
    pub children: Vec<usize>, // index in nodes
}

// impl TransformComponent {
//     fn from_gltf(g_node: &gltf::Node<'_>) -> Self {
//         let (position, r, scale) = g_node.transform().decomposed();
//         let quat = Quat::new(r[3], r[0], r[1], r[2]);
//         let euler = Euler::from(quat);
//         let rotation = [euler.x.0, euler.y.0, euler.z.0];
//         Self {
//             position,
//             rotation,
//             scale,
//         }
//     }
// }

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

fn extract_indices<'a, F>(reader: &Reader<'a, 'a, F>, indices: &mut Vec<u32>)
where
    F: Clone + Fn(gltf::Buffer<'a>) -> Option<&'a [u8]>,
{
    let base_vertex = indices.len() as u32;
    if let Some(read_indices) = reader.read_indices() {
        for i in read_indices.into_u32() {
            indices.push(base_vertex + i)
        }
    } else {
        // non indexed primitive
        let count = reader.read_positions().expect("position missing").len() as u32;
        indices.extend(base_vertex..base_vertex + count);
    }
}

fn extract_vertices<'a, F>(
    reader: &Reader<'a, 'a, F>,
    indices: &Vec<u32>,
    vertices: &mut Vec<MeshVertexData>,
) where
    F: Clone + Fn(gltf::Buffer<'a>) -> Option<&'a [u8]>,
{
    let positions = reader.read_positions().expect("Missing positions");

    for position in positions {
        let normal = [0.0, 1.0, 0.0];
        let uv = [0.0, 0.0];
        let tangent = [0.0, 0.0, 0.0, 0.0];

        vertices.push(MeshVertexData {
            position,
            normal,
            tangent,
            uv,
        });
    }

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
        generate_mikktspace_tangents(vertices, &indices);
    }
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

// step per load gltf
//     (LoadedScene)
//          |
//          Y
//   create Materialid
//          |
//          Y
//   crate Meshid
//          |
//          Y
//   spawn_scene ECS

pub fn load_gltf<P: AsRef<Path>>(
    path: P,
    asset_mgr: &mut AssetManager,
) -> Result<LoadedScene, ImportError> {
    if path.as_ref().extension().unwrap() != "gltf" {
        error!("File: {} is not a glTF", path.as_ref().display());
        return Err(ImportError::MeshLoadFailed);
    }

    let (gltf, buffers, _) = gltf::import(path.as_ref())?;

    let images: Vec<gltf::Image<'_>> = gltf.images().collect();

    let mut meshes = Vec::new();
    let mut materials = Vec::new();
    for g_mesh in gltf.meshes() {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut submeshes = Vec::new();
        for primitive in g_mesh.primitives() {
            assert_eq!(primitive.mode(), Mode::Triangles);

            let reader = primitive.reader(|b| Some(&buffers[b.index()]));
            let base_vertex = vertices.len() as u32;
            let index_start = indices.len() as u32;

            extract_indices(&reader, &mut indices);
            extract_vertices(&reader, &indices, &mut vertices);

            let index_end = indices.len() as u32;

            let mat_pbr = create_material(&primitive.material(), &images, &path);
            let material_id = mat_pbr_to_id(asset_mgr, &mat_pbr);
            materials.push(material_id);

            submeshes.push(SubMesh {
                index_range: index_start..index_end,
                base_vertex,
                material: material_id,
            });
        }

        let mesh_key = MeshKey {
            source: crate::assets::MeshSource::File {
                path: path.as_ref().into(),
                index: g_mesh.index(),
            },
        };

        let mesh_id = asset_mgr.meshes.get_or_create(mesh_key, || MeshDesc {
            vertices,
            indices,
            submeshes,
            bounds: extract_bbox(&g_mesh),
        });
        meshes.push(mesh_id);
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

fn resolve_image_uri(uri: &str, base_dir: &Path) -> Option<std::path::PathBuf> {
    Some(base_dir.join(uri))
}

fn path_from_ginfo(
    info: gltf::texture::Info<'_>,
    base_dir: &Path,
    images: &[gltf::Image<'_>],
) -> Option<std::path::PathBuf> {
    let image = images.get(info.texture().index())?;

    match image.source() {
        gltf::image::Source::Uri { uri, .. } => resolve_image_uri(uri, base_dir),
        _ => None,
    }
}

fn path_from_gtexture(
    texture: gltf::texture::Texture<'_>,
    base_dir: &Path,
) -> Option<std::path::PathBuf> {
    match texture.source().source() {
        gltf::image::Source::Uri { uri, .. } => resolve_image_uri(uri, base_dir),
        _ => None,
    }
}

fn create_material<P: AsRef<Path>>(
    gltf_material: &gltf::Material,
    images: &Vec<gltf::Image<'_>>,
    path: P,
) -> MaterialPBR {
    use material_manager::MaterialTextureSlot::*;

    let name = gltf_material.name().unwrap_or("material_no_name");
    let parent_path = path.as_ref().parent().unwrap_or_else(|| Path::new(""));

    // gltf pbr material
    let pbr = gltf_material.pbr_metallic_roughness();

    let mut material_pbr = MaterialPBR::default();

    material_pbr.name = name.into();
    material_pbr.base_color_factor = pbr.base_color_factor().into();
    material_pbr.roughness_factor = pbr.roughness_factor();
    material_pbr.metallic_factor = pbr.metallic_factor();
    material_pbr.emissive_factor = Vec3::from(gltf_material.emissive_factor()).extend(0.0);

    if let Some(normal_tex) = gltf_material.normal_texture() {
        material_pbr.normal_scale = normal_tex.scale();
        material_pbr.set_path(
            Normal,
            path_from_gtexture(normal_tex.texture(), parent_path),
        );
    }
    if let Some(occl_tex) = gltf_material.occlusion_texture() {
        material_pbr.occlusion_strength = occl_tex.strength().clamp(0.0, 1.0);
        material_pbr.set_path(
            Occlusion,
            path_from_gtexture(occl_tex.texture(), parent_path),
        )
    }
    if let Some(color_info) = pbr.base_color_texture() {
        material_pbr.set_path(BaseColor, path_from_ginfo(color_info, parent_path, &images));
    }
    if let Some(met_rough_info) = pbr.metallic_roughness_texture() {
        material_pbr.set_path(
            MetallicRoughness,
            path_from_ginfo(met_rough_info, parent_path, &images),
        );
    }
    if let Some(emissive_info) = gltf_material.emissive_texture() {
        material_pbr.set_path(
            Emissive,
            path_from_ginfo(emissive_info, parent_path, &images),
        );
    }

    material_pbr
}

fn mat_pbr_to_id(asset_mgr: &mut AssetManager, mat_pbr: &MaterialPBR) -> MaterialId {
    let mut mat_key = MaterialKey::default();

    for slot in material_manager::MaterialTextureSlot::ALL {
        if let Some(path) = mat_pbr.get_path(slot) {
            let key = TextureKey::File {
                color_space: slot.color_space().into(),
                path: path.into(),
                usage: slot.into(),
            };
            let desc = super::TextureDesc::File {
                key,
                sampler: super::SamplerDesc::Linear,
                mipmaps: false,
            };
            let id = asset_mgr.textures.get_or_create(desc);
            mat_key.textures[slot as usize] = Some(id);
        }
    }

    asset_mgr
        .materials
        .get_or_create(mat_key.clone(), || MaterialDesc {
            key: mat_key,
            emissive_factor: mat_pbr.emissive_factor,
            base_color_factor: mat_pbr.base_color_factor,
            metallic_factor: mat_pbr.metallic_factor,
            roughness_factor: mat_pbr.roughness_factor,
            normal_scale: mat_pbr.normal_scale,
            occlusion_strength: mat_pbr.occlusion_strength,
            use_texture_slot: mat_pbr.use_texture_slot,
        })
}



pub fn spawn_scene(world: &mut legion::World, loaded: &LoadedScene, asset_mgr: &AssetManager) {
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
            let mesh_id = &loaded.meshes[mesh_idx];
            let mut entry = world.entry(entity).unwrap();

            // MeshComponent
            entry.add_component(MeshComponent {
                handle: mesh_id.clone(),
            });

            // BoundingBoxComponent
            if let Some(mesh) = asset_mgr.meshes.get(*mesh_id){
                let bbox = &mesh.bounds;
                entry.add_component(BoundingBoxComponent {
                    bounding_box: bbox.clone(),
                    global_bounding_box: bbox.clone(),
                });

            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_load_scene() {
        let path = "./assets/cube/cube.gltf";
        let mut asset_mgr = AssetManager::default();
        
        let e = load_gltf(path, &mut asset_mgr);
        
        assert!(e.is_ok());
        assert_eq!(e.ok().iter().len(), 1);
    }

    #[test]
    fn should_create_material() {
        use material_manager::MaterialTextureSlot::*;
        
        let mut asset_mgr = AssetManager::default();
        
        let path = "./assets/cube/cube.gltf";
        let base_path = Path::new(path).parent().unwrap_or_else(|| Path::new(""));
        let color_path = base_path.join("Cube_BaseColor.png");
        let normal_path = base_path.join("Cube_normal.png");
        let mut mat_pbr = MaterialPBR::default();
        mat_pbr.name = "Cube".into();
        mat_pbr.set_path(BaseColor, Some(color_path));
        mat_pbr.set_path(Normal, Some(normal_path));
        mat_pbr.metallic_factor = 0.0;
        mat_pbr.roughness_factor = 1.0;
        
        let e = load_gltf(path, &mut asset_mgr).unwrap();
        
        assert_eq!(e.materials.len(), 1);
    }
    
    #[test]
    fn should_create_entity(){
        let mut world = legion::World::default();
        let mut asset_mgr = AssetManager::default();
        let path = "./assets/cube/cube.gltf";
        let loaded = load_gltf(path, &mut asset_mgr).unwrap();

        assert!(world.is_empty());

        spawn_scene(&mut world, &loaded, &asset_mgr);

        assert_eq!(world.len(), 1);

        use legion::query::IntoQuery;

        let mut query = <(&TagComponent, &MeshComponent)>::query();
        let (tag, mesh) = query.iter(&world).next().unwrap();

        let mesh = asset_mgr.meshes.get(mesh.handle).unwrap();
        
        assert_eq!(tag.name, "Cube");
        assert_eq!(mesh.indices.len(), 36);

    }
}
