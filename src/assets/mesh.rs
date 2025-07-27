use gltf::buffer;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshVertexData {
    position: [f32; 3],
    normal: [f32; 3],
    color: [f32; 3],
}

impl MeshVertexData {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 =>Float32x3, 1 => Float32x3, 2 => Float32x3];

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
}

pub struct Mesh {
    pub submeshes: Vec<SubMesh>,
}

#[allow(dead_code)]
pub fn load_gltf(
    device: &wgpu::Device,
    path: &std::path::Path,
) -> Result<Mesh, Box<dyn std::error::Error>> {
    if path.extension().unwrap_or_default() != "gltf" {
        return Err("File is not a glTF file".into());
    }

    let (gltf, buffers, _) = gltf::import(path)?;

    Ok(read_meshes(device, &gltf, buffers))
}

fn read_meshes(device: &wgpu::Device, gltf: &gltf::Document, buffers: Vec<buffer::Data>) -> Mesh {
    let mut submeshes: Vec<SubMesh> = Vec::new();
    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            let mesh = read_mesh(device, &primitive, buffers.clone());
            submeshes.push(mesh);
        }
    }
    Mesh { submeshes }
}

fn read_mesh(
    device: &wgpu::Device,
    primitive: &gltf::Primitive,
    buffers: Vec<buffer::Data>,
) -> SubMesh {
    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
    let positions = reader
        .read_positions()
        .expect("primitives must have the POSITION attribute ");
    let indices = reader
        .read_indices()
        .expect("primitives must have the INDICES attribute ");

    let mut vertices: Vec<MeshVertexData> = positions
        .map(|position| MeshVertexData {
            position,
            normal: [0.0, 1.0, 0.0],
            color: [1.0, 1.0, 1.0],
        })
        .collect();

    if let Some(normals) = reader.read_normals() {
        normals.enumerate().for_each(|(i, normal)| {
            vertices[i].normal = normal;
        });
    }

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    SubMesh {
        vertices,
        indices: indices.into_u32().collect(),
        vertex_buffer: Some(vertex_buffer),
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
        let (_adapter, device, _queue) = pollster::block_on(async {
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

        let result = load_gltf(&device, std::path::Path::new("./assets/cube/cube.gltf"));

        assert!(result.is_ok());
        let mesh = result.unwrap();

        assert_eq!(mesh.submeshes.len(), 1);
        assert_eq!(mesh.submeshes[0].indices.len(), 36);
    }
}
