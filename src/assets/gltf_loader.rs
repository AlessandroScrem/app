use gltf::mesh::{Mode, Reader};
use legion::EntityStore;
use std::path::Path;

use crate::{
    BoundingBoxComponent, GlobalModelComponent, HierarchyComponent, TagComponent,
    TransformComponent,
    assets::{
        MaterialDesc, MaterialId, MeshDesc, MeshId, MeshKey, SubMesh, asset_manager::AssetManager,
        vertexdata::MeshVertexData,
    },
    math::*,
    prelude::*,
};

pub struct LoadedScene {
    pub meshes: Vec<MeshId>,
    _materials: Vec<MaterialId>,
    pub nodes: Vec<NodeData>,
    _roots: Vec<usize>, // indici dei nodi root
}

pub struct NodeData {
    pub name: String,
    pub local_transform: TransformComponent,
    pub mesh: Option<usize>,  // index in meshes
    pub children: Vec<usize>, // index in nodes
}

pub fn generate_mikktspace_tangents(
    vertices: &mut [MeshVertexData],
    indices: &[u32],
    base_vertex: u32,
) {
    use mikktspace::{Geometry, generate_tangents};

    debug_assert!(
        indices
            .iter()
            .all(|&i| i - base_vertex < vertices.len() as u32)
    );

    struct Mikkt<'a> {
        vertices: &'a mut [MeshVertexData],
        indices: Vec<u32>,
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
            orientation: bool,
            face: usize,
            vert: usize,
        ) {
            let sign = if orientation { 1.0 } else { -1.0 };
            let idx = self.indices[face * 3 + vert] as usize;

            self.vertices[idx].tangent = [tangent[0], tangent[1], tangent[2], sign];
        }
    }

    let local_indices: Vec<u32> = indices.iter().map(|i| i - base_vertex).collect();

    let mut geom = Mikkt {
        vertices,
        indices: local_indices,
    };

    generate_tangents(&mut geom);
}

fn extract_indices<'a, F>(reader: &Reader<'a, 'a, F>, indices: &mut Vec<u32>, base_vertex: u32)
where
    F: Clone + Fn(gltf::Buffer<'a>) -> Option<&'a [u8]>,
{
    if let Some(read_indices) = reader.read_indices() {
        for i in read_indices.into_u32() {
            indices.push(base_vertex + i);
        }
    } else {
        let count = reader.read_positions().expect("position missing").len() as u32;
        indices.extend(base_vertex..base_vertex + count);
    }
}

fn extract_vertices<'a, F>(reader: &Reader<'a, 'a, F>, vertices: &mut Vec<MeshVertexData>) -> usize
where
    F: Clone + Fn(gltf::Buffer<'a>) -> Option<&'a [u8]>,
{
    let base = vertices.len();
    let positions = reader.read_positions().expect("Missing positions");

    for position in positions {
        vertices.push(MeshVertexData {
            position,
            normal: [0.0, 1.0, 0.0],
            tangent: [0.0; 4],
            uv: [0.0; 2],
        });
    }

    let count = vertices.len() - base;

    if let Some(normals) = reader.read_normals() {
        for (i, normal) in normals.enumerate() {
            vertices[base + i].normal = normal;
        }
    }

    if let Some(uvs) = reader.read_tex_coords(0) {
        for (i, uv) in uvs.into_f32().enumerate() {
            vertices[base + i].uv = uv;
        }
    }

    if let Some(tangents) = reader.read_tangents() {
        for (i, t) in tangents.enumerate() {
            vertices[base + i].tangent = t;
        }
    }

    count
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

#[allow(dead_code)]
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
            let index = primitive.indices().expect("indices not found");
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
            let index_start = indices.len();

            // assert che non ci siano set UV > 0
            use gltf::mesh::Semantic;
            debug_assert!(
                primitive.attributes().all(|(semantic, _)| {
                    match semantic {
                        Semantic::TexCoords(set) => set == 0,
                        _ => true,
                    }
                }),
                "Primitive {} set UV > 0 not yet supported",
                primitive.index()
            );

            let vertex_count = extract_vertices(&reader, &mut vertices);
            extract_indices(&reader, &mut indices, base_vertex);

            let index_end = indices.len();

            if reader.read_tangents().is_none() {
                let vertex_slice =
                    &mut vertices[base_vertex as usize..base_vertex as usize + vertex_count];

                generate_mikktspace_tangents(
                    vertex_slice,
                    &indices[index_start..index_end],
                    base_vertex,
                );
            }

            let material_id = create_material(&primitive.material(), asset_mgr, &images, &path);
            materials.push(material_id);

            submeshes.push(SubMesh {
                index_range: index_start as u32..index_end as u32,
                base_vertex,
                material: material_id,
            });
        }

        let mesh_key = MeshKey {
            source: crate::assets::MeshSource::File {
                path: path.as_ref().into(),
                submesh_index: g_mesh.index(),
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
        _materials: materials,
        meshes,
        nodes,
        _roots: roots,
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
    asset_mgr: &mut AssetManager,
    images: &Vec<gltf::Image<'_>>,
    path: P,
) -> MaterialId {
    use material_asset::MaterialTextureSlot::*;
    let texture_asset = &mut asset_mgr.textures;

    let name = gltf_material.name().unwrap_or("material_no_name");
    let parent_path = path.as_ref().parent().unwrap_or_else(|| Path::new(""));

    // gltf pbr material
    let pbr = gltf_material.pbr_metallic_roughness();

    let mut material_desc = MaterialDesc::default();

    material_desc.set_name(name);
    material_desc.base_color_factor = pbr.base_color_factor().into();
    material_desc.roughness_factor = pbr.roughness_factor();
    material_desc.metallic_factor = pbr.metallic_factor();
    material_desc.emissive_factor = Vec3::from(gltf_material.emissive_factor()).extend(0.0);

    if let Some(normal_tex) = gltf_material.normal_texture() {
        material_desc.normal_scale = normal_tex.scale();
        material_desc.set_texture(
            texture_asset,
            Normal,
            path_from_gtexture(normal_tex.texture(), parent_path),
        );
    }
    if let Some(occl_tex) = gltf_material.occlusion_texture() {
        material_desc.occlusion_strength = occl_tex.strength().clamp(0.0, 1.0);
        material_desc.set_texture(
            texture_asset,
            Occlusion,
            path_from_gtexture(occl_tex.texture(), parent_path),
        )
    }
    if let Some(color_info) = pbr.base_color_texture() {
        material_desc.set_texture(
            texture_asset,
            BaseColor,
            path_from_ginfo(color_info, parent_path, &images),
        );
    }
    if let Some(met_rough_info) = pbr.metallic_roughness_texture() {
        material_desc.set_texture(
            texture_asset,
            MetallicRoughness,
            path_from_ginfo(met_rough_info, parent_path, &images),
        );
    }
    if let Some(emissive_info) = gltf_material.emissive_texture() {
        material_desc.set_texture(
            texture_asset,
            Emissive,
            path_from_ginfo(emissive_info, parent_path, &images),
        );
    }

    asset_mgr
        .materials
        .get_or_create(material_desc.key.clone(), || material_desc)
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

            if let Some(mut entry) = world.entry(entity) {
                // MeshComponent
                entry.add_component(MeshComponent {
                    handle: mesh_id.clone(),
                });

                // BoundingBoxComponent
                if let Some(mesh) = asset_mgr.meshes.get(*mesh_id) {
                    let bbox = &mesh.bounds;
                    entry.add_component(BoundingBoxComponent {
                        bounding_box: bbox.clone(),
                        global_bounding_box: bbox.clone(),
                    });
                }
            }
        }
    }

    // 3️⃣ collega la gerarchia
    for (i, node) in loaded.nodes.iter().enumerate() {
        let parent = node_to_entity[i];

        for &child_idx in &node.children {
            let child = node_to_entity[child_idx];

            world.entry_mut(parent).ok().map(|mut e| {
                e.get_component_mut::<HierarchyComponent>()
                    .map(|h| h.children.push(child))
            });

            world.entry_mut(child).ok().map(|mut e| {
                e.get_component_mut::<HierarchyComponent>()
                    .map(|h| h.parent = Some(parent))
            });
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
        use material_asset::MaterialTextureSlot::*;

        let mut asset_mgr = AssetManager::default();
        let texture_asset = &mut asset_mgr.textures;

        let path = "./assets/cube/cube.gltf";
        let base_path = Path::new(path).parent().unwrap_or_else(|| Path::new(""));
        let color_path = base_path.join("Cube_BaseColor.png");
        let normal_path = base_path.join("Cube_normal.png");
        let mut mat_desc = MaterialDesc::default();
        mat_desc.set_name("Cube");
        mat_desc.set_texture(texture_asset, BaseColor, Some(color_path));
        mat_desc.set_texture(texture_asset, Normal, Some(normal_path));
        mat_desc.metallic_factor = 0.0;
        mat_desc.roughness_factor = 1.0;

        let e = load_gltf(path, &mut asset_mgr).unwrap();

        assert_eq!(e._materials.len(), 1);
    }

    #[test]
    fn should_create_entity() {
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
