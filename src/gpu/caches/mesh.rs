use crate::{
    assets::{MeshId, MeshVertexData},
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
    fn retain<F>(&mut self, contains: F) 
    where
        F: Fn(&MeshId) -> bool
    {
        // Sync cleanup
        self.map.retain(|id, mesh| {
            let keep = contains(&id);
            if !keep {
                //remove id
                // update stats
                self.stats.remove(mesh.estimated_size);
                trace!("removed gpu mesh {:?}", id);
            }
            
            keep
        });
    }

    pub fn sync(&mut self, device: &wgpu::Device, inputs: &[SyncInput<MeshId, MeshDesc>]) {
        let desired: HashSet<_> = inputs.iter().map(|i| i.id).collect();

        self.retain(|id| desired.contains(id));

        for input in inputs {
            if !self.map.contains_key(input.id) {
                self.create_gpu_mesh(input.id, input.data, device);
            }
        }
    }
    
    fn create_gpu_mesh(&mut self, id: MeshId, mesh_desc: &MeshDesc, device: &wgpu::Device) {
        let vertices = &mesh_desc.vertices;
        let indices = &mesh_desc.indices;
        let mesh = create_gpu_mesh(device, vertices, indices);
        self.stats.add(mesh.estimated_size);
        self.map.insert(id, mesh);
    }

    pub fn get(&self, id: &MeshId) -> Option<&GpuMesh> {
        self.map.get(*id)
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
