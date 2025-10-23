use std::{path::Path, time::Instant};
use wgpu::util::DeviceExt;

use crate::{
    prelude::*,
    assets::{
        material_manager::{Material, MaterialManager},
        texture_manager::TextureManager,
        vertexdata::MeshVertexData,
    },
    entities::bounding_box::BoundingBox,
    renderer::gpu_manager::{GPUResourceManager, LayoutKind},
    math::*,
};

fn compute_global_transforms(nodes: &mut [Node]) {
    fn compute_global_recursive(nodes: &mut [Node], index: usize, parent_transform: Mat4) {
        let local = nodes[index].local_transform;
        let global = parent_transform * local;
        nodes[index].global_transform = global;

        // visita i figli
        let children = nodes[index].children.clone();
        for child_id in children {
            compute_global_recursive(nodes, child_id, global);
        }
    }
    // i nodi root sono quelli senza parent
    let roots: Vec<usize> = nodes
        .iter()
        .filter(|n| n.parent.is_none())
        .map(|n| n.index)
        .collect();

    for root_id in roots {
        compute_global_recursive(nodes, root_id, Mat4::identity());
    }
}

fn assign_parents(nodes: &mut [Node]) {
    for i in 0..nodes.len() {
        let children = nodes[i].children.clone(); // copia perché ci serve iterare
        for &child_id in &children {
            if let Some(child) = nodes.get_mut(child_id) {
                child.parent = Some(i);
            }
        }
    }
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
    _world: &mut legion::World,
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

    info!("--\t gltf import is {} ms", timer.elapsed().as_millis());

    let mut nodes: Vec<Node> = document
        .nodes()
        .map(|g_node| Node::from_gltf(&g_node))
        .collect();

    // 1. Collega i genitori
    assign_parents(&mut nodes);
    // 2. Calcola i global transform
    compute_global_transforms(&mut nodes);

    let mut meshes: Vec<Mesh> = document
        .meshes()
        .map(|gltf_mesh| {
            Mesh::from_gltf(
                &gltf_mesh,
                buffers.clone(),
                &images,
                path,
                material_manager,
                texture_manager,
                gpu_resource_manager,
                device,
            )
        })
        .collect();

    let mesh = meshes.remove(0);

    Ok(mesh)
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
    use std::sync::Arc;

    #[test]
    fn should_load_mesh() {
        let (device, queue) = crate::test_utils::get_device_and_queue();

        let gpu_manager = GPUResourceManager::new(&device);
        let gpu_manager = Arc::new(gpu_manager);
        let mut material_manager = MaterialManager::new(device.clone(), gpu_manager.clone());

        let mut texture_manager = TextureManager::new(device.clone(), queue.clone());

        let mut world = legion::World::default();

        let result = load_gltf(
            &mut world,
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

    #[test]
    fn assign_parent_to_nodes() {
        let path = std::path::Path::new("./assets/Lantern/Lantern.gltf");
        let (document, _buffers, _) = gltf::import(path).unwrap();

        let mut nodes: Vec<Node> = document
            .nodes()
            .map(|g_node| Node::from_gltf(&g_node))
            .collect();

        assert_eq!(nodes.len(), 4);

        print_gltf_document(&document);

        // 1. Collega i genitori
        assign_parents(&mut nodes);

        let child0 = &nodes[0];
        let child1 = &nodes[1];
        let child2 = &nodes[2];
        let parent = &nodes[3];

        assert_eq!(child0.parent.unwrap(), parent.index);
        assert_eq!(child1.parent.unwrap(), parent.index);
        assert_eq!(child2.parent.unwrap(), parent.index);

        assert_eq!(parent.children, [child0.index, child1.index, child2.index]);
    }

    #[test]
    fn compute_global_trasforms_to_child_nodes() {
        let path = std::path::Path::new("./assets/Lantern/Lantern.gltf");
        let (document, _buffers, _) = gltf::import(path).unwrap();

        let mut nodes: Vec<Node> = document
            .nodes()
            .map(|g_node| Node::from_gltf(&g_node))
            .collect();

        assert_eq!(nodes.len(), 4);

        // 1. Collega i genitori
        assign_parents(&mut nodes);
        // 2. Calcola i global transform
        compute_global_transforms(&mut nodes);

        let child0 = &nodes[0];
        let child1 = &nodes[1];
        let child2 = &nodes[2];
        let parent = &nodes[3];
        let parent_transform = parent.local_transform;

        // childs
        assert_eq!(
            child0.global_transform,
            parent_transform * child0.local_transform
        );
        assert_eq!(
            child1.global_transform,
            parent_transform * child1.local_transform
        );
        assert_eq!(
            child2.global_transform,
            parent_transform * child2.local_transform
        );

        // parent global_transform reflect local_transform
        assert_eq!(parent.global_transform, parent.local_transform);
    }
}
