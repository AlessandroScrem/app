use std::path::Path;

use crate::{
    assets::{
        material_manager::MaterialManager, mesh::*, mesh_manager::MeshManager,
        texture_manager::TextureManager,
    },
    renderer::gpu_manager::GpuManager,
};

/// A function to help create a mesh entity.
pub fn create_from_gltf<P: AsRef<Path>>(
    path: P,
    cmd: &mut legion::systems::CommandBuffer,
    device: &wgpu::Device,
    mesh_manager: &mut MeshManager,
    material_manager: &mut MaterialManager,
    texture_manager: &mut TextureManager,
    gpu_manager: &GpuManager,
) {
    match load_gltf(
        cmd,
        mesh_manager,
        material_manager,
        texture_manager,
        gpu_manager,
        device,
        path.as_ref(),
    ) {
        Err(e) => {
            log::error!("glTF import failed: {}", e);
        }
        _ => {}
    }
}
