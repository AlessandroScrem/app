use std::{path::Path, sync::Arc};

use crate::{
    assets::{material_manager::MaterialManager, mesh::*, texture_manager::TextureManager},
    renderer::gpu_manager::GPUResourceManager,
};

use legion::*;

/// A function to help create a mesh entity.
pub fn create(world: &mut World, resources: &Resources) {
    let device = resources.get::<wgpu::Device>().unwrap();
    let mut material_manager = resources.get_mut::<MaterialManager>().unwrap();
    let mut texture_manager = resources.get_mut::<TextureManager>().unwrap();
    let gpu_resource_manager = resources.get::<Arc<GPUResourceManager>>().unwrap();

    load_gltf(
        world,
        &mut material_manager,
        &mut texture_manager,
        &gpu_resource_manager,
        &device,
        // Path::new("./assets/cube/cube.gltf"),
        Path::new("C:/Users/aless/Downloads/glTF-Sample-Models/2.0/Lantern/glTF/Lantern.gltf"),
    );

    load_gltf(
        world,
        &mut material_manager,
        &mut texture_manager,
        &gpu_resource_manager,
        &device,
        Path::new("./assets/avocado/avocado.gltf"),
    );
}
