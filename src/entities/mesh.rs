use std::path::Path;

use crate::{
    assets::{
        material_manager::MaterialManager, mesh::*, mesh_manager::MeshManager,
        texture_manager::TextureManager,
    },
    renderer::gpu_manager::GpuManager,
};

use legion::*;

/// A function to help create a mesh entity.
pub fn create_from_gltf<P: AsRef<Path>>(path: P, world: &mut World, resources: &Resources) {
    let device = resources.get::<wgpu::Device>().unwrap();
    let mut mesh_manager = resources.get_mut::<MeshManager>().unwrap();
    let mut material_manager = resources.get_mut::<MaterialManager>().unwrap();
    let mut texture_manager = resources.get_mut::<TextureManager>().unwrap();
    let gpu_resource_manager = resources.get::<GpuManager>().unwrap();

    match load_gltf(
        world,
        &mut mesh_manager,
        &mut material_manager,
        &mut texture_manager,
        &gpu_resource_manager,
        &device,
        path.as_ref(),
    ) {
        Err(e) => {
            log::error!("glTF import failed: {}", e);
        }
        _ => {}
    }
}
