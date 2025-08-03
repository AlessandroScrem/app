
use std::{path::{Path}, sync::Arc};

use crate::{assets::{material_manager::MaterialManager, mesh::*}, resources::{self, gpu_manager::{self, GPUResourceManager}}};
use legion::*;

/// A function to help create a mesh entity.
pub fn create(world: &mut World, resources: &Resources, path: &Path) -> Entity {
     let device = resources.get::<wgpu::Device>().unwrap();
     let mut material_manager = resources.get_mut::<MaterialManager>().unwrap();

     // let mut gpu_manager = resources.get_mut::<GPUResourceManager>().unwrap();
     // prova(&mut gpu_manager);

     let mesh = load_gltf(&mut material_manager, &device, path).unwrap();

     world.push((Arc::new(mesh), ))
}


// fn prova(gpu_manager: &mut GPUResourceManager) {

// }


