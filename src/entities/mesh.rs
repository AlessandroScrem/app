use std::{path::Path, sync::Arc};

use crate::{
    BoundingBoxComponent, MeshComponent, TagComponent, TransformComponent,
    assets::{material_manager::MaterialManager, mesh::*, texture_manager::TextureManager},
    entities::bounding_box::BoundingBox,
    renderer::{gpu_manager::GPUResourceManager, uniform::ModelUniform},
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
            world,
            &mut material_manager,
            &mut texture_manager,
            &gpu_resource_manager,
            &device,
            Path::new("./assets/avocado/avocado.gltf"),
        )
        .expect("unable_load mesh");

        let transform = TransformComponent {
            position: [2.0, 0.0, 0.0],
            scale: [10.0f32, 10.0, 10.0],
            ..Default::default()
        };
        let bounding_box = (mesh.vmin, mesh.vmax).into();
        let vertex_buffer = BoundingBox::create_vertex_buffer(&device, &bounding_box, &transform);
        let model_uniform = ModelUniform::new(transform.compute_model_matrix());

        let _entity = world.push((
            TagComponent {
                name: mesh.name.clone(),
            },
            transform.clone(),
            MeshComponent { data: mesh },
            BoundingBoxComponent {
                bounding_box,
                vertex_buffer,
            },
            model_uniform,
        ));
    }

    {
        let mesh = load_gltf(
            world,
            &mut material_manager,
            &mut texture_manager,
            &gpu_resource_manager,
            &device,
            // Path::new("./assets/cube/cube.gltf"),
            Path::new("C:/Users/aless/Downloads/glTF-Sample-Models/2.0/Lantern/glTF/Lantern.gltf"),
        )
        .expect("unable_load mesh");

        let bounding_box = (mesh.vmin, mesh.vmax).into();
        let transform = TransformComponent::default();
        let vertex_buffer = BoundingBox::create_vertex_buffer(&device, &bounding_box, &transform);
        let model_uniform = ModelUniform::new(transform.compute_model_matrix());

        let _entity = world.push((
            TagComponent {
                name: mesh.name.clone(),
            },
            MeshComponent { data: mesh },
            transform.clone(),
            BoundingBoxComponent {
                bounding_box,
                vertex_buffer,
            },
            model_uniform,
        ));

    }
}
