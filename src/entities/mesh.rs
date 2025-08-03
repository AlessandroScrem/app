
use std::{path::{Path}, sync::Arc};

use crate::assets::{material_manager::MaterialManager, mesh::*};
use legion::*;

/// A function to help create a mesh entity.
pub fn create(world: &mut World, resources: &Resources, path: &Path) -> Entity {
     let device = resources.get::<wgpu::Device>().unwrap();
     let mut material_manager = resources.get_mut::<MaterialManager>().unwrap();

     let mesh = load_gltf(&mut material_manager, &device, path).unwrap();

     world.push((Arc::new(mesh), ))
}



