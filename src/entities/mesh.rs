use std::{path::Path, sync::Arc};

use crate::{
    assets::{material_manager::MaterialManager, mesh::*},
    resources::gpu_manager::GPUResourceManager,
    transform::Transform,
};
use legion::*;

/// A function to help create a mesh entity.
pub fn create(world: &mut World, resources: &Resources) {
    let device = resources.get::<wgpu::Device>().unwrap();
    let mut material_manager = resources.get_mut::<MaterialManager>().unwrap();
    let gpu_resource_manager = resources.get::<Arc<GPUResourceManager>>().unwrap();

    let avocado = load_gltf(
        &mut material_manager,
        &gpu_resource_manager,
        &device,
        Path::new("./assets/avocado/avocado.gltf"),
    )
    .expect("unable_load mesh");
    world.push((
        avocado,
        Transform {
            position: [2.0, 0.0, 0.0],
            scale: [10.0f32, 10.0, 10.0],
            ..Default::default()
        },
    ));

    let cube = load_gltf(
        &mut material_manager,
        &gpu_resource_manager,
        &device,
        Path::new("./assets/cube/cube.gltf"),
    )
    .expect("unable_load mesh");
    world.push((cube, Transform::default()));
}
