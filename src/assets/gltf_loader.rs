use super::*;
use gltf::mesh::{Mode, Reader};
use std::{path::Path, time::Instant};

use crate::{
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

// public wrapper manage error messages
pub fn load_gltf<P: AsRef<Path>>(path: P, asset_mgr: &mut AssetManager) -> Option<LoadedScene> {
    match load_gltf_internal(&path, asset_mgr) {
        Ok(scene) => Some(scene),
        Err(e) => {
            warn!("Failed to load glTF {}: {}", path.as_ref().display(), e);
            None
        }
    }
}

// step for load gltf
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
fn load_gltf_internal<P: AsRef<Path>>(
    path: P,
    asset_mgr: &mut AssetManager,
) -> Result<LoadedScene, ImportError> {
    let timer = Instant::now();
    let (gltf, buffers, _) = gltf::import(path.as_ref())?;

    info!("Import gltf took: {:?}", timer.elapsed());

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
            debug_assert!(
                primitive.attributes().all(|(semantic, _)| {
                    use gltf::mesh::Semantic::TexCoords;
                    match semantic {
                        TexCoords(set) => set == 0,
                        _ => true,
                    }
                }),
                "Primitive {} set UV > 0 not yet supported",
                primitive.index()
            );

            let vertex_count = extract_vertices(&reader, &mut vertices)?;
            extract_indices(&reader, &mut indices, base_vertex, vertex_count);

            let index_end = indices.len();

            if reader.read_tangents().is_none() {
                trace!("generate tangent for {:?}", primitive.material().name());
                generate_mikktspace_tangents(&mut vertices, &indices[index_start..index_end]);
            }

            let material_id = create_material(&primitive.material(), asset_mgr, &path);
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

    // print_gltf_document(&gltf);
    info!("loading gltf took: {:?}", timer.elapsed());

    Ok(scene)
}

fn generate_mikktspace_tangents(vertices: &mut [MeshVertexData], indices: &[u32]) {
    use mikktspace::{Geometry, generate_tangents};

    debug_assert!(indices.iter().all(|&i| (i as usize) < vertices.len()));

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
            let v = &mut self.vertices[idx];

            v.tangent[0] += tangent[0];
            v.tangent[1] += tangent[1];
            v.tangent[2] += tangent[2];

            // w = ±1 senza accumulo
            // FIX: invert w
            v.tangent[3] = -sign;
        }
    }

    let mut geom = Mikkt {
        vertices,
        indices: indices.to_vec(),
    };

    let result = generate_tangents(&mut geom);
    debug_assert!(result, "mikkspace: failed to genetate tangents");
}

fn extract_indices<'a, F>(
    reader: &Reader<'a, 'a, F>,
    indices: &mut Vec<u32>,
    base_vertex: u32,
    vertex_count: usize,
) where
    F: Clone + Fn(gltf::Buffer<'a>) -> Option<&'a [u8]>,
{
    if let Some(read_indices) = reader.read_indices() {
        for i in read_indices.into_u32() {
            indices.push(base_vertex + i);
        }
    } else {
        warn!("Missing Indices, recalculating from vertices ...");
        indices.extend(base_vertex..base_vertex + vertex_count as u32);
    }
}

fn extract_vertices<'a, F>(
    reader: &Reader<'a, 'a, F>,
    vertices: &mut Vec<MeshVertexData>,
) -> Result<usize, ImportError>
where
    F: Clone + Fn(gltf::Buffer<'a>) -> Option<&'a [u8]>,
{
    let base = vertices.len();

    let positions = reader.read_positions().ok_or_else(|| {
        warn!("Gltf: missing POSITION");
        ImportError::MissingPositions
    })?;

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

    Ok(count)
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

#[allow(unused)]
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

// remove %20 from uri, gltf spec requires that spaces in uri are encoded as %20, but we want to support unencoded spaces in file paths
fn url_decoding(uri: &str) -> String {
    uri.replace("%20", " ")
}

fn path_from_ginfo(info: gltf::texture::Info<'_>, base_dir: &Path) -> Option<std::path::PathBuf> {
    match info.texture().source().source() {
        gltf::image::Source::Uri { uri, .. } => Some(base_dir.join(url_decoding(uri))),
        _ => None,
    }
}

fn path_from_gtexture(
    texture: gltf::texture::Texture<'_>,
    base_dir: &Path,
) -> Option<std::path::PathBuf> {
    match texture.source().source() {
        gltf::image::Source::Uri { uri, .. } => Some(base_dir.join(url_decoding(uri))),
        _ => None,
    }
}

fn create_material<P: AsRef<Path>>(
    gltf_material: &gltf::Material,
    asset_mgr: &mut AssetManager,
    path: P,
) -> MaterialId {
    use material_asset::MaterialTextureSlot::*;
    let texture_asset = &mut asset_mgr.textures;

    let name = gltf_material.name().unwrap_or_default();
    let parent_path = path.as_ref().parent().unwrap_or_else(|| Path::new(""));

    // gltf pbr material
    let pbr = gltf_material.pbr_metallic_roughness();

    let mut material_desc = MaterialDesc::default();

    let alpha_mode = match gltf_material.alpha_mode() {
        gltf::material::AlphaMode::Blend => material_asset::AlphaMode::Blend,
        gltf::material::AlphaMode::Mask => gltf_material
            .alpha_cutoff()
            .map(|alpha_cutoff| AlphaMode::Mask { alpha_cutoff })
            .unwrap_or(AlphaMode::mask_default()),
        gltf::material::AlphaMode::Opaque => material_asset::AlphaMode::Opaque,
    };

    material_desc.set_name(name);
    material_desc.alpha_mode = alpha_mode;
    material_desc.base_color_factor = pbr.base_color_factor().into();
    material_desc.roughness_factor = pbr.roughness_factor();
    material_desc.metallic_factor = pbr.metallic_factor();
    material_desc.emissive_factor = Vec3::from(gltf_material.emissive_factor()).extend(0.0);
    material_desc.ior = gltf_material.ior().unwrap_or(1.5);

    if let Some(color_info) = pbr.base_color_texture() {
        material_desc.set_texture(
            texture_asset,
            BaseColor,
            path_from_ginfo(color_info, parent_path),
        );
    }
    if let Some(normal_tex) = gltf_material.normal_texture() {
        material_desc.normal_scale = normal_tex.scale();
        material_desc.set_texture(
            texture_asset,
            Normal,
            path_from_gtexture(normal_tex.texture(), parent_path),
        );
    }
    if let Some(met_rough_info) = pbr.metallic_roughness_texture() {
        material_desc.set_texture(
            texture_asset,
            MetallicRoughness,
            path_from_ginfo(met_rough_info, parent_path),
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
    if let Some(emissive_info) = gltf_material.emissive_texture() {
        material_desc.set_texture(
            texture_asset,
            Emissive,
            path_from_ginfo(emissive_info, parent_path),
        );
    }
    if let Some(transmission) = gltf_material.transmission() {
        let factor = transmission.transmission_factor();
        material_desc.transmission = Some(assets::Transmission { factor });

        if let Some(transmission_texture) = transmission.transmission_texture() {
            material_desc.set_texture(
                texture_asset,
                Transmission,
                path_from_ginfo(transmission_texture, parent_path),
            );
        }
    }

    if let Some(volume) = gltf_material.volume() {
        material_desc.volume = Some(assets::Volume {
            thickness_factor: volume.thickness_factor(),
            attenuation_distance: volume.attenuation_distance(),
            attenuation_color: volume.attenuation_color(),
        });

        if let Some(volume_texture) = volume.thickness_texture() {
            material_desc.set_texture(
                texture_asset,
                Volume,
                path_from_ginfo(volume_texture, parent_path),
            );
        }
    }

    // println!("Metarial created {:#?}", material_desc);
    asset_mgr.materials.get_or_create(material_desc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_load_scene() {
        let path = "./assets/cube/cube.gltf";
        let mut asset_mgr = AssetManager::default();

        let e = load_gltf_internal(path, &mut asset_mgr);

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

        entities::spawn_scene(&mut world, &loaded, &asset_mgr);

        assert_eq!(world.len(), 1);

        use legion::query::IntoQuery;

        let mut query = <(&TagComponent, &MeshComponent)>::query();
        let (tag, mesh) = query.iter(&world).next().unwrap();

        let mesh = asset_mgr.meshes.get(mesh.handle).unwrap();

        assert_eq!(tag.name, "Cube");
        assert_eq!(mesh.indices.len(), 36);
    }
}
