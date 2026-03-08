use crate::{assets::{MeshAssets, MeshId, MeshVertexData}, renderer::{GpuResourceStats, HasGpuStats}, uniform::ModelUniform};

use super::*;
use slotmap::SecondaryMap;
use wgpu::util::DeviceExt;

#[derive(Default)]
pub struct GpuMeshCache {
    map: SecondaryMap<MeshId, GpuMesh>,
    stats: GpuResourceStats,
}

impl HasGpuStats for GpuMeshCache {
    fn get_stats(&self) -> GpuResourceStats {
        self.stats.clone()
    }
}

impl GpuMeshCache {
    pub fn ensure(
        &mut self,
        id: MeshId,
        assets: &MeshAssets,
        gpu_mgr: &GpuManager,
        device: &wgpu::Device,
    ) {
        if !self.map.contains_key(id) {
            let mesh = Self::create_gpu_mesh(id, assets, gpu_mgr, device);
            self.stats.add(mesh.estimated_size);
            self.map.insert(id, mesh);
        }
    }

    pub fn retain(&mut self, assets: &MeshAssets) {
        // Sync cleanup
        self.map.retain(|id, mesh| {
            if assets.contains_key(id) {
                true //mantain
            } else {
                // update stats
                self.stats.remove(mesh.estimated_size);
                trace!("removed gpu mesh {:?}", id);
                false // remove
            }
        });
    }

    pub fn create_gpu_mesh(
        id: MeshId,
        asset: &MeshAssets,
        gpu_manager: &GpuManager,
        device: &wgpu::Device,
    ) -> GpuMesh {
        let mesh = asset.get(id).unwrap();
        let vertices = &mesh.vertices;
        let indices = &mesh.indices;
        create_gpu_mesh(device, gpu_manager, vertices, indices)
    }

    pub fn update(&self, id: &MeshId, queue: &wgpu::Queue, uniform: &ModelUniform) {
        if let Some(mesh) = self.map.get(*id) {
            let buffer = &mesh.model_uniform;
            queue.write_buffer(buffer, 0, bytemuck::bytes_of(uniform));
        }
    }

    pub fn get(&self, id: &MeshId) -> Option<&GpuMesh> {
        self.map.get(*id)
    }

    pub fn keys(&self) -> impl Iterator<Item = MeshId> {
        self.map.keys()
    }
}

pub struct GpuMesh {
    pub vertexbuffer: wgpu::Buffer,
    pub indexbuffer: wgpu::Buffer,
    _indexcount: u32,
    pub model_bind_group: wgpu::BindGroup,
    pub model_uniform: wgpu::Buffer,
    estimated_size: usize,
}

fn create_gpu_mesh(
    device: &wgpu::Device,
    gpu_manager: &GpuManager,
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
    let estimated_size = vertices.len() + indices.len() + size_of::<ModelUniform>();

    GpuMesh {
        vertexbuffer,
        indexbuffer,
        model_uniform,
        model_bind_group,
        _indexcount: indexcount,
        estimated_size,
    }
}
