use legion::*;

use crate::{
    assets::{MeshAsset, asset_manager::AssetManager, gltf_loader::LoadedScene},
    ecs::components::*,
};

pub struct Scene {
    pub world: World,
    pub schedule: Schedule,
}

impl Default for Scene {
    fn default() -> Self {
        let world = World::default();

        let mut schedule_builder = Schedule::builder();
        let schedule = schedule_builder.build();

        Scene { world, schedule }
    }
}

pub fn spawn_scene_transform(
    world: &mut legion::World,
    loaded: &LoadedScene,
    asset_mgr: &AssetManager,
    root_transform: TransformComponent,
) {
    let mut node_to_entity = Vec::with_capacity(loaded.nodes.len());

    // 1️⃣ crea tutte le entity
    for (i, node) in loaded.nodes.iter().enumerate() {
        // Il primo nodo è il root virtuale
        let transform = if i == 0 {
            root_transform.clone()
        } else {
            node.local_transform.clone()
        };

        let entity = world.push((
            TagComponent {
                name: node.name.clone(),
            },
            transform,
            HierarchyComponent::default(),
            GlobalModelComponent::default(),
        ));

        node_to_entity.push(entity);
    }

    // 2️⃣ assegna mesh + material
    for (i, node) in loaded.nodes.iter().enumerate() {
        if let Some(mesh_idx) = node.mesh {
            let entity = node_to_entity[i];
            let mesh_id = &loaded.meshes[mesh_idx];

            if let Some(mut entry) = world.entry(entity) {
                entry.add_component(MeshComponent {
                    handle: mesh_id.clone(),
                });

                if let Some(mesh_asset) = asset_mgr.get::<MeshAsset>(*mesh_id) {
                    let bbox = &mesh_asset.desc.bounds;
                    entry.add_component(BoundingBoxComponent {
                        bounding_box: bbox.clone(),
                        global_bounding_box: bbox.clone(),
                    });
                }
            }
        }
    }

    // 3️⃣ collega la gerarchia
    for (i, node) in loaded.nodes.iter().enumerate() {
        let parent = node_to_entity[i];

        for &child_idx in &node.children {
            let child = node_to_entity[child_idx];

            if let Ok(mut entry) = world.entry_mut(parent) {
                if let Ok(h) = entry.get_component_mut::<HierarchyComponent>() {
                    h.children.push(child);
                }
            }

            if let Ok(mut entry) = world.entry_mut(child) {
                if let Ok(h) = entry.get_component_mut::<HierarchyComponent>() {
                    h.parent = Some(parent);
                }
            }
        }
    }
}
