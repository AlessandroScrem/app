use std::path::Path;

use gltf::buffer;
use wgpu::util::DeviceExt;

use crate::{
    assets::{
        material_manager::{Material, MaterialManager},
        texture_manager::TextureManager,
    },
    renderer::gpu_manager::{GPUResourceManager, LayoutKind},
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

pub struct SubMesh {
    pub vertices: Vec<MeshVertexData>,
    pub indices: Vec<u32>,
    pub(crate) vertex_buffer: Option<wgpu::Buffer>,
    pub(crate) index_buffer: Option<wgpu::Buffer>,
    pub(crate) index_count: usize,
    pub primitive_topology: wgpu::PrimitiveTopology,
    pub material: Material,
}

pub struct Mesh {
    pub name: String,
    pub submeshes: Vec<SubMesh>,
    pub model_uniform_buffer: wgpu::Buffer,
    pub model_bind_group: wgpu::BindGroup,
}

#[allow(dead_code)]
pub fn load_gltf(
    material_manager: &mut MaterialManager,
    texture_manager: &mut TextureManager,
    gpu_resource_manager: &GPUResourceManager,
    device: &wgpu::Device,
    path: &Path,
) -> Result<Mesh, Box<dyn std::error::Error>> {
    let timer = std::time::Instant::now();

    if path.extension().unwrap_or_default() != "gltf" {
        return Err("File is not a glTF file".into());
    }

    let (document, buffers, _) = gltf::import(path)?;
    let images: Vec<gltf::Image<'_>> = document.images().collect();

    println!("--\t gltf import is {} ms", timer.elapsed().as_millis());

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

        println!(
            "--\t create material: {} is {} ms",
            primitive.material().name().unwrap_or("no_name"),
            timer.elapsed().as_millis()
        );

        let primitive_topology = get_primitive_mode(primitive.mode());

        let submesh = SubMesh {
            vertices,
            indices,
            vertex_buffer: Some(vertex_buffer),
            index_buffer: Some(index_buffer),
            index_count,
            primitive_topology,
            material,
        };
        submeshes.push(submesh);
    }

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

    println!(
        "Loading mesh {} took {} ms",
        path.display(),
        timer.elapsed().as_millis()
    );

    Ok(Mesh {
        name,
        submeshes,
        model_uniform_buffer,
        model_bind_group,
    })
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
    use crate::renderer::gpu_manager::GPUResourceManager;
    use std::sync::Arc;

    #[test]
    fn should_load_mesh() {
        let (device, queue) = crate::get_device_and_queue();

        let gpu_manager = GPUResourceManager::new(&device);
        let gpu_manager = Arc::new(gpu_manager);
        let mut material_manager = MaterialManager::new(device.clone(), gpu_manager.clone());

        let mut texture_manager = TextureManager::new(device.clone(), queue.clone());

        let result = load_gltf(
            &mut material_manager,
            &mut texture_manager,
            &gpu_manager,
            &device,
            std::path::Path::new("./assets/cube/cube.gltf"),
        );

        assert!(result.is_ok());
        let mesh = result.unwrap();

        assert_eq!(mesh.submeshes.len(), 1);
        assert_eq!(mesh.submeshes[0].indices.len(), 36);
    }
}
