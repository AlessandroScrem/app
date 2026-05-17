use crate::{
    assets::{MeshAssets, MeshId, MeshVertexData},
    renderer::{GpuResourceStats, HasGpuStats},
};

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
    pub fn ensure(&mut self, id: MeshId, assets: &MeshAssets, device: &wgpu::Device) {
        if !self.map.contains_key(id) {
            let mesh = Self::create_gpu_mesh(id, assets, device);
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

    pub fn create_gpu_mesh(id: MeshId, asset: &MeshAssets, device: &wgpu::Device) -> GpuMesh {
        let mesh = asset.get(id).unwrap();
        let vertices = &mesh.vertices;
        let indices = &mesh.indices;
        create_gpu_mesh(device, vertices, indices)
    }

    pub fn get(&self, id: &MeshId) -> Option<&GpuMesh> {
        self.map.get(*id)
    }

    #[allow(unused)]
    pub fn keys(&self) -> impl Iterator<Item = MeshId> {
        self.map.keys()
    }
}

pub struct GpuMesh {
    pub vertexbuffer: wgpu::Buffer,
    pub indexbuffer: wgpu::Buffer,
    _indexcount: u32,
    estimated_size: usize,
}

fn create_gpu_mesh(
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
