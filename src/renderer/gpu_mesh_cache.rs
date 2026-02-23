use super::*;
use slotmap::SecondaryMap;
use wgpu::util::DeviceExt;

#[derive(Default)]
pub(crate) struct GpuMeshCache {
    map: SecondaryMap<MeshId, GpuMesh>,
}

impl GpuMeshCache {
    pub(crate) fn ensure(
        &mut self,
        id: MeshId,
        assets: &MeshAssets,
        gpu_mgr: &GpuManager,
        device: &wgpu::Device,
    ) {
        if !self.map.contains_key(id) {
            let value = Self::create_gpu_mesh(id, assets, gpu_mgr, device);
            self.map.insert(id, value);
        }
    }

    pub(crate) fn retain(&mut self, assets: &MeshAssets) {
        // Sync cleanup
        self.map.retain(|id, _| assets.contains_key(id));
    }

    pub(crate) fn create_gpu_mesh(
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

    pub(crate) fn update(&self, id: &MeshId, queue: &wgpu::Queue, uniform: &ModelUniform) {
        if let Some(mesh) = self.map.get(*id) {
            let buffer = &mesh.model_uniform;
            queue.write_buffer(buffer, 0, bytemuck::bytes_of(uniform));
        }
    }

    pub(crate) fn remove(&mut self, id: &MeshId) {
        if self.map.contains_key(*id) {
            self.map.remove(*id);
        }
    }

    pub(crate) fn get(&self, id: &MeshId) -> Option<&GpuMesh> {
        self.map.get(*id)
    }

    pub(crate) fn keys(&self) -> impl Iterator<Item = MeshId> {
        self.map.keys()
    }

    pub(crate) fn values(&self) -> impl Iterator<Item = &GpuMesh> {
        self.map.values()
    }
}

pub(crate) struct GpuMesh {
    pub(crate) vertexbuffer: wgpu::Buffer,
    pub(crate) indexbuffer: wgpu::Buffer,
    pub(crate) indexcount: u32,
    pub(crate) model_bind_group: wgpu::BindGroup,
    pub(crate) model_uniform: wgpu::Buffer,
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

    GpuMesh {
        vertexbuffer,
        indexbuffer,
        model_uniform,
        model_bind_group,
        indexcount,
    }
}
