use std::path::Path;

use gltf::buffer;
use wgpu::util::DeviceExt;

use crate::assets::material_manager::{Material, MaterialManager};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshVertexData {
    position: [f32; 3],
    normal: [f32; 3],
    color: [f32; 3],
    uv: [f32; 2],
}

impl MeshVertexData {
    const ATTRIBS: [wgpu::VertexAttribute; 4] =
        wgpu::vertex_attr_array![0 =>Float32x3, 1 => Float32x3, 2 => Float32x3, 3 => Float32x2];

    pub fn get_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct SubMesh {
    pub vertices: Vec<MeshVertexData>,
    pub indices: Vec<u32>,
    pub(crate) vertex_buffer: Option<wgpu::Buffer>,
    pub(crate) index_buffer: Option<wgpu::Buffer>,
    pub(crate) index_count: usize,
    pub primitive_topology: wgpu::PrimitiveTopology,
}

pub struct Mesh {
    pub name: String,
    pub submeshes: Vec<SubMesh>,
}

#[allow(dead_code)]
pub fn load_gltf(
    material_manager: &mut MaterialManager,
    device: &wgpu::Device,
    path: &Path,
) -> Result<Mesh, Box<dyn std::error::Error>> {
    if path.extension().unwrap_or_default() != "gltf" {
        return Err("File is not a glTF file".into());
    }

    let (document, buffers, _) = gltf::import(path)?;
    let images: Vec<gltf::Image<'_>> = document.images().collect();

    let gltf_mesh = document.meshes().next().expect("mesh [0] not present");
    let name = gltf_mesh.name().unwrap_or("mesh").to_string();

    let mut submeshes: Vec<SubMesh> = Vec::new();

    for primitive in gltf_mesh.primitives() {
        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

        let positions = reader
            .read_positions()
            .expect("primitives must have the POSITION attribute ");
        let indices = reader
            .read_indices()
            .expect("primitives must have the INDICES attribute ");
        let indices: Vec<u32> = indices.into_u32().collect();

        let mut vertices: Vec<MeshVertexData> = positions
            .map(|position| MeshVertexData {
                position,
                normal: [0.0, 1.0, 0.0],
                color: [0.5, 0.5, 0.5],
                uv: [0.0, 0.0],
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
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: &bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let index_count = indices.len();

        let gltf_material: gltf::Material<'_> = primitive.material();
        let pbr = gltf_material.pbr_metallic_roughness();

        // materials
        let color_factor = pbr.base_color_factor();
        let color = cgmath::Vector4::new(
            color_factor[0],
            color_factor[1],
            color_factor[2],
            color_factor[3],
        );

        let main_info = pbr.base_color_texture();
        let mut normal_texture = None;
        let normals_texture = gltf_material.normal_texture();
        if normals_texture.is_some() {
            let normal_source = normals_texture.unwrap().texture().source().source();
            match normal_source {
                gltf::image::Source::Uri { uri, .. } => {
                    let texture_file_name = Some(
                        Path::new(&uri)
                            .file_name()
                            .and_then(std::ffi::OsStr::to_str)
                            .unwrap()
                            .to_string(),
                    );
                    if texture_file_name.is_some() {
                        normal_texture = Some(texture_file_name.unwrap());
                    }
                }
                _ => (),
            }
        }
        let roughness_info = pbr.metallic_roughness_texture();
        let roughness = pbr.roughness_factor();
        let metallic = pbr.metallic_factor();

        let main_texture = get_texture_url(&main_info, &images);
        let roughness_texture = get_texture_url(&roughness_info, &images);

        let has_pbr_texture = roughness_texture.is_some();

        let material = Material {
            main_texture: main_texture.unwrap_or("white.png".to_string()),
            normal_texture: normal_texture.unwrap_or("white.png".to_string()),
            roughness_texture: roughness_texture.unwrap_or("white.png".to_string()),
            roughness,
            metallic,
            roughness_override: if has_pbr_texture { 0.0 } else { 1.0 },
            metallic_override: if has_pbr_texture { 0.0 } else { 1.0 },
            color,
            textures: std::collections::HashMap::new(),
        };

        material_manager.add_material(material, path.to_path_buf());

        let primitive_topology = get_primitive_mode(primitive.mode());

        let submesh = SubMesh {
            vertices,
            indices,
            vertex_buffer: Some(vertex_buffer),
            index_buffer: Some(index_buffer),
            index_count,
            primitive_topology,
        };
        submeshes.push(submesh);
    }

    Ok(Mesh { name, submeshes })
}

fn get_texture_url(
    info: &Option<gltf::texture::Info<'_>>,
    images: &Vec<gltf::Image<'_>>,
) -> Option<String> {
    let mut file_name = None;
    if info.is_some() {
        let info = info.as_ref().unwrap();
        let tex = info.texture();

        let image: Option<&gltf::Image<'_>> = images.get(tex.index());
        if image.is_some() {
            let image = image.unwrap();
            let source = image.source();
            match source {
                gltf::image::Source::Uri { uri, .. } => {
                    let texture_file_name = Some(Path::new(&uri).to_str().unwrap().to_string());
                    if texture_file_name.is_some() {
                        file_name = Some(texture_file_name.unwrap());
                    }
                }
                _ => (),
            }
        }
    }
    file_name
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

#[allow(dead_code)]
fn print_tree(node: &gltf::Node, depth: i32) {
    for _ in 0..(depth - 1) {
        print!("  ");
    }
    print!(" -");
    print!(" Node {}", node.index());
    print!(" ({})", node.name().unwrap_or("<Unnamed>"));
    println!();

    for child in node.children() {
        print_tree(&child, depth + 1);
    }
}

#[allow(dead_code)]
fn print_meshes(gltf: &gltf::Document, buffers: Vec<buffer::Data>) {
    for mesh in gltf.meshes() {
        println!("Mesh #{}", mesh.index());
        for primitive in mesh.primitives() {
            let index = primitive.indices().unwrap();
            println!(
                "- Primitive #{} Index Count {}",
                primitive.index(),
                index.count()
            );

            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            if let Some(iter) = reader.read_positions() {
                for vertex_position in iter {
                    println!("{:?}", vertex_position);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::resources::gpu_manager::GPUResourceManager;
    use std::sync::Arc;

    use super::*;

    #[test]
    fn should_load_mesh() {
        let (_adapter, device, queue) = pollster::block_on(async {
            let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::None,
                    compatible_surface: None,
                    ..Default::default()
                })
                .await
                .unwrap();

            let (device, queue) = adapter
                .request_device(&wgpu::DeviceDescriptor::default())
                .await
                .unwrap();

            let arc_device = Arc::new(device);
            let arc_queue = Arc::new(queue);

            (adapter, arc_device, arc_queue)
        });

        let gpu_manager = GPUResourceManager::new(&device);
        let mut material_manager =
            MaterialManager::new(device.clone(), queue, Arc::new(gpu_manager));

        let result = load_gltf(
            &mut material_manager,
            &device,
            std::path::Path::new("./assets/cube/cube.gltf"),
        );

        assert!(result.is_ok());
        let mesh = result.unwrap();

        assert_eq!(mesh.submeshes.len(), 1);
        assert_eq!(mesh.submeshes[0].indices.len(), 36);
    }
}
