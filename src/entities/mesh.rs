use std::{path::Path, sync::Arc};

use crate::{
    TransformComponent, assets::{material_manager::MaterialManager, mesh::*, texture_manager::TextureManager}, renderer::gpu_manager::GPUResourceManager
};

use legion::*;

/// A function to help create a mesh entity.
pub fn create(world: &mut World, resources: &Resources) {
    let device = resources.get::<wgpu::Device>().unwrap();
    let mut material_manager = resources.get_mut::<MaterialManager>().unwrap();
    let mut texture_manager = resources.get_mut::<TextureManager>().unwrap();
    let gpu_resource_manager = resources.get::<Arc<GPUResourceManager>>().unwrap();

    // if let Some(e) = load_gltf(
    //     world,
    //     &mut material_manager,
    //     &mut texture_manager,
    //     &gpu_resource_manager,
    //     &device,
    //     Path::new("./assets/Lantern/Lantern.gltf"),
    // ) {
    //     if let Ok(mut entry) = world.entry_mut(e) {
    //         let transform = entry.get_component_mut::<TransformComponent>().unwrap();
    //         transform.position[1] += 1.0;
    //     }
    // }

    // if let Some(e) = load_gltf(
    //     world,
    //     &mut material_manager,
    //     &mut texture_manager,
    //     &gpu_resource_manager,
    //     &device,
    //     Path::new("./assets/cube/cube.gltf"),
    // ) {
    //     if let Ok(mut entry) = world.entry_mut(e) {
    //         let transform = entry.get_component_mut::<TransformComponent>().unwrap();
    //         transform.scale = [30.0, 1.0, 30.0];
    //     }
    // }

    if let Some(e) = load_gltf(
        world,
        &mut material_manager,
        &mut texture_manager,
        &gpu_resource_manager,
        &device,
        Path::new("C:/Users/aless/Downloads/glTF-Sample-Models/2.0/DamagedHelmet/glTF/DamagedHelmet.gltf"),
    ) {
        if let Ok(mut entry) = world.entry_mut(e) {
            let transform = entry.get_component_mut::<TransformComponent>().unwrap();
            transform.position = [15.0, 10.0, 0.0];
            transform.scale = [20.0, 20.0, 20.0];
        }
    }

}

