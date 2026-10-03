use std::collections::HashMap;

use crate::ResourceId;
use crate::{
    assets::MeshVertexData,
    gpu::{GpuResourceStats, HasGpuStats},
};

use wgpu::util::DeviceExt;

#[derive(Default)]
pub struct GpuMeshCache {
    map: HashMap<ResourceId, GpuMesh>,
    stats: GpuResourceStats,
}

impl HasGpuStats for GpuMeshCache {
    fn get_stats(&self) -> GpuResourceStats {
        self.stats.clone()
    }
}

impl GpuMeshCache {
    pub fn insert(&mut self, id: ResourceId, gpu_mesh: GpuMesh) {
        self.stats.add(gpu_mesh.estimated_size);
        self.map.insert(id, gpu_mesh);
    }

    pub fn get(&self, id: &ResourceId) -> Option<&GpuMesh> {
        self.map.get(id)
    }

    #[allow(unused)]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn remove(&mut self, id: ResourceId) {
        if let Some(gpu_mesh) = self.map.remove(&id) {
            self.stats.remove(gpu_mesh.estimated_size);
        }
    }
}

pub struct GpuMesh {
    pub vertexbuffer: wgpu::Buffer,
    pub indexbuffer: wgpu::Buffer,
    _indexcount: u32,
    estimated_size: usize,
}

impl GpuMesh {
    pub fn new(
        device: &wgpu::Device,
        vertices: &Vec<MeshVertexData>,
        indices: &Vec<u32>,
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

        let indexcount = indices.len() as u32;
        let estimated_size = vertices.len() + indices.len();

        GpuMesh {
            vertexbuffer,
            indexbuffer,
            _indexcount: indexcount,
            estimated_size,
        }
    }
}
