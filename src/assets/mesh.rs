use std::path::{Path, PathBuf};

use gltf::buffer;
use wgpu::util::DeviceExt;

use crate::{
    assets::texture::Texture,
    resources::gpu_manager::{GPUResourceManager},
};

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

pub struct PBRMaterial {
    pub main_texture: String,
    pub roughness_texture: String,
    pub normal_texture: String,
    pub roughness: f32,
    pub metallic: f32,
    pub roughness_override: f32,
    pub metallic_override: f32,
    pub color: cgmath::Vector4<f32>,
    pub textures: std::collections::HashMap<String, Texture>,
}

impl PBRMaterial {}

pub struct SubMesh {
    pub vertices: Vec<MeshVertexData>,
    pub indices: Vec<u32>,
    pub(crate) vertex_buffer: Option<wgpu::Buffer>,
    pub(crate) index_buffer: Option<wgpu::Buffer>,
    pub(crate) index_count: usize,
    pub material: PBRMaterial,
    pub primitive_topology: wgpu::PrimitiveTopology,
}

pub struct Mesh {
    pub name: String,
    pub submeshes: Vec<SubMesh>,
}

#[allow(dead_code)]
pub fn load_gltf(
    gpu_manager: &mut GPUResourceManager,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    path: &Path,
) -> Result<Mesh, Box<dyn std::error::Error>> {
    let relative_path = path.parent().unwrap();

    if path.extension().unwrap_or_default() != "gltf" {
        return Err("File is not a glTF file".into());
    }

    let (document, buffers, _) = gltf::import(path)?;
    Ok(read_meshes(
        gpu_manager,
        device,
        queue,
        &document,
        buffers,
        relative_path,
    ))
}

fn read_meshes(
    gpu_manager: &mut GPUResourceManager,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    document: &gltf::Document,
    buffers: Vec<buffer::Data>,
    path: &Path,
) -> Mesh {
    let mut submeshes: Vec<SubMesh> = Vec::new();
    let mut name = String::new();
    let images: Vec<gltf::Image<'_>> = document.images().collect();

    for gltf_mesh in document.meshes() {
        name = gltf_mesh.name().unwrap_or("mesh").to_string();

        for primitive in gltf_mesh.primitives() {
            let mesh = read_mesh(
                gpu_manager,
                device,
                &queue,
                &primitive,
                buffers.clone(),
                &images,
                path,
            );
            submeshes.push(mesh);
        }
    }
    Mesh { name, submeshes }
}

fn read_mesh(
    gpu_manager: &mut GPUResourceManager,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    primitive: &gltf::Primitive,
    buffers: Vec<buffer::Data>,
    images: &Vec<gltf::Image<'_>>,
    path: &Path,
) -> SubMesh {
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
    let roughness_info = pbr.metallic_roughness_texture();
    let roughness = pbr.roughness_factor();
    let metallic = pbr.metallic_factor();

    let main_texture = get_texture_url(&main_info, &images);
    let roughness_texture = get_texture_url(&roughness_info, &images);

    let has_pbr_texture = roughness_texture.is_some();

    let mut material = PBRMaterial {
        main_texture: main_texture.unwrap_or("white.png".to_string()),
        normal_texture: String::new(),
        roughness_texture: String::new(),
        roughness,
        metallic,
        roughness_override: if has_pbr_texture { 0.0 } else { 1.0 },
        metallic_override: if has_pbr_texture { 0.0 } else { 1.0 },
        color,
        textures: std::collections::HashMap::new(),
    };

    let texture = get_texture(path.join(&material.main_texture), device, queue);

    let texture_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

    let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&texture.sampler),
            },
        ],
        label: Some("diffuse_bind_group"),
    });

    gpu_manager.add_texture_bind_group(texture_bind_group_layout, diffuse_bind_group);

    material
        .textures
        .insert(material.main_texture.clone(), texture);

    let primitive_topology = get_primitive_mode(primitive.mode());

    SubMesh {
        vertices,
        indices,
        vertex_buffer: Some(vertex_buffer),
        index_buffer: Some(index_buffer),
        index_count,
        material,
        primitive_topology,
    }
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

fn get_texture(
    path: PathBuf,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> super::texture::Texture {
    let buffer = std::fs::read(&path).unwrap();

    super::texture::Texture::new(device, queue, buffer)
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
                .request_device(&wgpu::DeviceDescriptor::default(), None)
                .await
                .unwrap();

            (adapter, device, queue)
        });

        let mut gpu_resource_manager = GPUResourceManager::new(&device);

        let result = load_gltf(
            &mut gpu_resource_manager,
            &device,
            &queue,
            std::path::Path::new("./assets/cube/cube.gltf"),
        );

        assert!(result.is_ok());
        let mesh = result.unwrap();

        assert_eq!(mesh.submeshes.len(), 1);
        assert_eq!(mesh.submeshes[0].indices.len(), 36);
    }
}
