use std::{path::Path, sync::Arc};

use crate::{
    BoundingBoxComponent, GlobalModelComponent, HierarchyComponent, MeshComponent, TagComponent,
    TransformComponent,
    assets::{material_manager::MaterialManager, mesh::*, texture_manager::TextureManager},
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
        let global_model = GlobalModelComponent::from(transform.compute_model_matrix());
        let bbox_component = BoundingBoxComponent::new(&device, bounding_box);

        let _entity = world.push((
            TagComponent {
                name: mesh.name.clone(),
            },
            transform.clone(),
            MeshComponent { data: mesh },
            bbox_component,
            global_model,
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
        let global_model = GlobalModelComponent::from(transform.compute_model_matrix());
        let bbox_component = BoundingBoxComponent::new(&device, bounding_box);

        let _entity = world.push((
            TagComponent {
                name: mesh.name.clone(),
            },
            MeshComponent { data: mesh },
            transform.clone(),
            bbox_component,
            global_model,
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
            let global_model = GlobalModelComponent::from(transform.compute_model_matrix());
            let bbox_component = BoundingBoxComponent::new(&device, bounding_box);
            let mesh_component = MeshComponent { data: mesh };

            world.push((
                TagComponent {
                    name: "Parent_cube".into(),
                },
                bbox_component,
                transform,
                global_model,
                mesh_component,
                HierarchyComponent {
                    parent: None,
                    children: Vec::new(),
                },
            ))
        };

        let child_entity1 = {
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
            let global_model = GlobalModelComponent::from(transform.compute_model_matrix());
            let bbox_component = BoundingBoxComponent::new(&device, bounding_box);
            // let bbox_component = BoundingBoxComponent {
            //     vertex_buffer: BoundingBox::create_vertex_buffer(
            //         &device,
            //         &bounding_box,
            //         &global_model.mat,
            //     ),
            //     bounding_box,
            // };
            let mesh_component = MeshComponent { data: mesh };

            world.push((
                TagComponent {
                    name: "Child_cube".into(),
                },
                bbox_component,
                transform,
                global_model,
                mesh_component,
                HierarchyComponent {
                    parent: Some(parent_entity.clone()),
                    children: Vec::new(),
                },
            ))
        };

        let child_entity2 = {
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
                position: [0.0, -4.0, 0.0],
                scale: [0.5f32, 0.5, 0.5],
                ..Default::default()
            };
            let global_model = GlobalModelComponent::from(transform.compute_model_matrix());
            let bbox_component = BoundingBoxComponent::new(&device, bounding_box);
            // let bbox_component = BoundingBoxComponent {
            //     vertex_buffer: BoundingBox::create_vertex_buffer(
            //         &device,
            //         &bounding_box,
            //         &global_model.mat,
            //     ),
            //     bounding_box,
            // };
            let mesh_component = MeshComponent { data: mesh };

            world.push((
                TagComponent {
                    name: "Child_cube2".into(),
                },
                bbox_component,
                transform,
                global_model,
                mesh_component,
                HierarchyComponent {
                    parent: Some(child_entity1.clone()),
                    children: Vec::new(),
                },
            ))
        };

        // assign children to parent
        if let Ok(mut entry) = world.entry_mut(parent_entity) {
            if let Ok(hierarchy) = entry.get_component_mut::<HierarchyComponent>() {
                hierarchy.children.push(child_entity1.clone());
            }
        }

        // assign children to child1
        if let Ok(mut entry) = world.entry_mut(child_entity1) {
            if let Ok(hierarchy) = entry.get_component_mut::<HierarchyComponent>() {
                hierarchy.children.push(child_entity2.clone());
            }
        }
    }
}
