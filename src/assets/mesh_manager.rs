use std::collections::HashMap;

use wgpu::util::DeviceExt as _;

use crate::{
    assets::{material_manager::MaterialId, vertexdata::MeshVertexData},
    renderer::{GpuManager, gpu_manager::LayoutKind},
};

pub struct MeshManager {
    meshes: HashMap<usize, GpuMesh>,
    id: usize,
}

pub struct GpuMesh {
    vertexbuffer: wgpu::Buffer,
    indexbuffer: wgpu::Buffer,
    indexcount: u32,
    material: MaterialId,
    model_bind_group: wgpu::BindGroup,
    model_uniform: wgpu::Buffer,
}

impl MeshManager {
    pub fn new() -> Self {
        Self {
            meshes: HashMap::new(),
            id: 0,
        }
    }

    pub fn add_mesh(&mut self, mesh: GpuMesh)->usize {
        let id = self.id;
        self.meshes.insert(id, mesh);
        self.id = id + 1;
        id
    }

    pub fn get_vertexbuffer(&self, id: usize) -> &wgpu::Buffer {
        &self
            .meshes
            .get(&id)
            .expect("Unable to get Mesh")
            .vertexbuffer
    }
    pub fn get_indexbuffer(&self, id: usize) -> &wgpu::Buffer {
        &self
            .meshes
            .get(&id)
            .expect("Unable to get Mesh")
            .indexbuffer
    }
    pub fn get_indexcount(&self, id: usize) -> u32 {
        self.meshes.get(&id).expect("Unable to get Mesh").indexcount
    }
    pub fn get_model_bindgroup(&self, id: usize) -> &wgpu::BindGroup {
        &self
            .meshes
            .get(&id)
            .expect("Unable to get Mesh")
            .model_bind_group
    }
    pub fn get_model_uniform(&self, id: usize) -> &wgpu::Buffer {
        &self
            .meshes
            .get(&id)
            .expect("Unable to get Mesh")
            .model_uniform
    }
    pub fn get_material(&self, id: usize) -> &MaterialId {
        &self.meshes.get(&id).expect("Unable to get Mesh").material
    }
}

pub fn create_mesh(
    device: &wgpu::Device,
    gpu_manager: &GpuManager,
    vertices: &Vec<MeshVertexData>,
    indices: &Vec<u32>,
    material: MaterialId,
) -> GpuMesh {
    let vertexbuffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Mesh Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let indexbuffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Mesh Index Buffer"),
        contents: &bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    let model_uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Model Uniform Buffer"),
        contents: bytemuck::cast_slice(&[crate::renderer::uniform::ModelUniform::default()]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let model_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &gpu_manager.get_layout(LayoutKind::Model),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: model_uniform.as_entire_binding(),
        }],
        label: Some("Model Bind Group"),
    });

    let indexcount = indices.len() as u32;

    GpuMesh {
        vertexbuffer,
        indexbuffer,
        material,
        model_uniform,
        model_bind_group,
        indexcount,
    }
}
