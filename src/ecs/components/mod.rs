pub(crate) mod bounding_box_impl;
pub mod components;
pub(crate) mod hierarchy;
pub(crate) mod light;

use crate::assets::gltf_loader::LoadedScene;
use crate::{assets::asset_manager::AssetManager, assets::mesh_asset::MeshAsset};
pub(crate) use components::*;

use legion::EntityStore;

pub(crate) fn enable_all_lights(enable: bool, world: &mut legion::World) {
    use legion::query::IntoQuery;

    let mut query = <&mut LightComponent>::query();

    for light in query.iter_mut(world) {
        light.enabled = enable;
    }
}

pub fn spawn_scene(world: &mut legion::World, loaded: &LoadedScene, asset_mgr: &AssetManager) {
    let mut node_to_entity = Vec::with_capacity(loaded.nodes.len());

    // 1️⃣ crea tutte le entity
    for node in &loaded.nodes {
        let name = node.name.clone();
        let entity = world.push((
            TagComponent { name },
            TransformComponent::from(node.local_transform.clone()),
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
                // MeshComponent
                entry.add_component(MeshComponent {
                    handle: mesh_id.clone(),
                });

                // BoundingBoxComponent
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

            world.entry_mut(parent).ok().map(|mut e| {
                e.get_component_mut::<HierarchyComponent>()
                    .map(|h| h.children.push(child))
            });

            world.entry_mut(child).ok().map(|mut e| {
                e.get_component_mut::<HierarchyComponent>()
                    .map(|h| h.parent = Some(parent))
            });
        }
    }
}
