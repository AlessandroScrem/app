
use std::{path::{Path}, sync::Arc};

use crate::assets::mesh::*;
use legion::*;

/// A function to help create a mesh entity.
pub fn create(world: &mut World, resources: &Resources, path: &Path) -> Entity {
     let device = resources.get::<wgpu::Device>().unwrap();
     let mesh = load_gltf(&device, path).unwrap();

     world.push((Arc::new(mesh), ))
}


