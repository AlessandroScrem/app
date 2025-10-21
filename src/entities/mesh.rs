use std::{path::Path, sync::Arc};

use crate::{
    BoundingBoxComponent, HierarchyComponent, MeshComponent, TagComponent, TransformComponent,
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

/// A function to help create a mesh hierarchy.
pub fn create_hirarchy(world: &mut World, resources: &Resources) {
    let device = resources.get::<wgpu::Device>().unwrap();
    let mut material_manager = resources.get_mut::<MaterialManager>().unwrap();
    let mut texture_manager = resources.get_mut::<TextureManager>().unwrap();
    let gpu_resource_manager = resources.get::<Arc<GPUResourceManager>>().unwrap();

    {
        let parent_entity = {
            let mesh = load_gltf(
                world,
                &mut material_manager,
                &mut texture_manager,
                &gpu_resource_manager,
                &device,
                Path::new("./assets/cube/cube.gltf"),
            )
            .expect("unable_load mesh");

            let bounding_box = (mesh.vmin, mesh.vmax).into();
            let transform = TransformComponent::default();
            let model_uniform = ModelUniform::new(transform.compute_model_matrix());
            let bbox_component = BoundingBoxComponent {
                vertex_buffer: BoundingBox::create_vertex_buffer(
                    &device,
                    &bounding_box,
                    &transform,
                ),
                bounding_box,
            };
            let mesh_component = MeshComponent { data: mesh };

            world.push((
                TagComponent {
                    name: "Parent_cube".into(),
                },
                bbox_component,
                transform,
                model_uniform,
                mesh_component,
                HierarchyComponent {
                    parent: None,
                    children: Vec::new(),
                },
            ))
        };

        let child_entity = {
            let mesh = load_gltf(
                world,
                &mut material_manager,
                &mut texture_manager,
                &gpu_resource_manager,
                &device,
                Path::new("./assets/cube/cube.gltf"),
            )
            .expect("unable_load mesh");

            let bounding_box = (mesh.vmin, mesh.vmax).into();

            let transform = TransformComponent {
                position: [0.0, -2.0, 0.0],
                scale: [0.5f32, 0.5, 0.5],
                ..Default::default()
            };
            let model_uniform = ModelUniform::new(transform.compute_model_matrix());
            let bbox_component = BoundingBoxComponent {
                vertex_buffer: BoundingBox::create_vertex_buffer(
                    &device,
                    &bounding_box,
                    &transform,
                ),
                bounding_box,
            };
            let mesh_component = MeshComponent { data: mesh };

            world.push((
                TagComponent {
                    name: "Child_cube".into(),
                },
                bbox_component,
                transform,
                model_uniform,
                mesh_component,
                HierarchyComponent {
                    parent: Some(parent_entity.clone()),
                    children: Vec::new(),
                },
            ))
        };

        // assign children to parent
        if let Ok(mut entry) = world.entry_mut(parent_entity) {
            if let Ok(hierarchy) = entry.get_component_mut::<HierarchyComponent>() {
                hierarchy.children.push(child_entity.clone());
            }
        }
    }
}
