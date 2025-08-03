
use std::{path::{Path}, sync::Arc};

use crate::{assets::mesh::*, resources::gpu_manager::{GPUResourceManager}};
use legion::*;

/// A function to help create a mesh entity.
pub fn create(world: &mut World, resources: &Resources, path: &Path) -> Entity {
     let device = resources.get::<wgpu::Device>().unwrap();
     let queue = resources.get_mut::<wgpu::Queue>().unwrap();
     let mut gpu_manager = resources.get_mut::<GPUResourceManager>().unwrap();
     let mesh = load_gltf(&mut gpu_manager, &device, &queue, path).unwrap();

     world.push((Arc::new(mesh), ))
}


