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
        let mesh = load_gltf(
            &mut material_manager,
            &mut texture_manager,
            &gpu_resource_manager,
            &device,
            Path::new("./assets/avocado/Avocado.gltf"),
        )
        .expect("unable_load mesh");

        let transform = TransformComponent {
            position: [2.0, 0.0, 0.0],
            scale: [10.0f32, 10.0, 10.0],
            ..Default::default()
        };
        let bounding_box = (mesh.vmin, mesh.vmax).into();
        let vertex_buffer = BoundingBox::create_vertex_buffer(&device, &bounding_box, &transform);

        world.push((
            TagComponent {
                name: mesh.name.clone(),
            },
            transform,
            MeshComponent { data: mesh },
            BoundingBoxComponent {
                bounding_box,
                vertex_buffer,
            },
        ));
    }

    {
        let mesh = load_gltf(
            &mut material_manager,
            &mut texture_manager,
            &gpu_resource_manager,
            &device,
            Path::new("./assets/cube/cube.gltf"),
        )
        .expect("unable_load mesh");

        let bounding_box = (mesh.vmin, mesh.vmax).into();
        let transform = TransformComponent::default();
        let vertex_buffer = BoundingBox::create_vertex_buffer(&device, &bounding_box, &transform);

        world.push((
            TagComponent {
                name: mesh.name.clone(),
            },
            MeshComponent { data: mesh },
            transform,
            BoundingBoxComponent {
                bounding_box,
                vertex_buffer,
            },
        ));
    }
}
