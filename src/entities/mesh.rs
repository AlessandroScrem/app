use std::{path::Path, sync::Arc};

use crate::{
    BoundingBoxComponent, MeshComponent, TagComponent, TransformComponent,
    assets::{material_manager::MaterialManager, mesh::*, texture_manager::TextureManager},
    entities::bounding_box::BoundingBox,
    renderer::gpu_manager::GPUResourceManager,
};

use legion::*;

/// A function to help create a mesh entity.
pub fn create(world: &mut World, resources: &Resources) {
    let device = resources.get::<wgpu::Device>().unwrap();
    let mut material_manager = resources.get_mut::<MaterialManager>().unwrap();
    let mut texture_manager = resources.get_mut::<TextureManager>().unwrap();
    let gpu_resource_manager = resources.get::<Arc<GPUResourceManager>>().unwrap();

    {
        let avocado = load_gltf(
            &mut material_manager,
            &mut texture_manager,
            &gpu_resource_manager,
            &device,
            Path::new("./assets/avocado/avocado.gltf"),
        )
        .expect("unable_load mesh");

        let bounding_box = BoundingBox {
            min: avocado.vmin,
            max: avocado.vmax,
        };
        let vertex_buffer = BoundingBox::create_vertex_buffer(&device, &bounding_box);

        world.push((
            TagComponent {
                name: avocado.name.clone(),
            },
            MeshComponent { data: avocado },
            TransformComponent {
                position: [2.0, 0.0, 0.0],
                scale: [10.0f32, 10.0, 10.0],
                ..Default::default()
            },
            BoundingBoxComponent {
                bounding_box,
                vertex_buffer,
            },
        ));
    }

    {
        let cube = load_gltf(
            &mut material_manager,
            &mut texture_manager,
            &gpu_resource_manager,
            &device,
            Path::new("./assets/cube/cube.gltf"),
        )
        .expect("unable_load mesh");

        let bounding_box = BoundingBox {
            min: cube.vmin,
            max: cube.vmax,
        };
        let vertex_buffer = BoundingBox::create_vertex_buffer(&device, &bounding_box);
        
        world.push((
            TagComponent {
                name: cube.name.clone(),
            },
            MeshComponent { data: cube },
            TransformComponent::default(),
            BoundingBoxComponent {
                bounding_box,
                vertex_buffer,
            },
        ));
    }
}
