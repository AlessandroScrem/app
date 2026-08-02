use super::components::*;
use crate::assets::GlobalAssetId;
use crate::assets::asset_manager::AssetManager;
use crate::prelude::warn;

use legion::*;

pub(crate) fn disable_entity(entity: Entity, world: &mut legion::World, disable: bool) {
    // if not root node do nothing
    if !is_root(entity, world) {
        warn!("{:?} Not Root: Add Parent abort", entity);
        return;
    }

    if let Some(mut e) = world.entry(entity) {
        if disable {
            e.add_component(Hidden);
        } else {
            e.remove_component::<Hidden>();
        }
    }
}

pub(crate) fn add_parent(entity: Entity, world: &mut legion::World) {
    // if not root node do nothing
    if !is_root(entity, world) {
        warn!("{:?} Not Root: Add Parent abort", entity);
        return;
    }

    // Add parent node and set entity as child
    let new_root = {
        world.push((
            TagComponent {
                name: "New Node".into(),
            },
            HierarchyComponent {
                parent: None,
                children: vec![entity],
            },
            GlobalModelComponent::default(),
            TransformComponent::default(),
        ))
    };

    // Register new node as root
    world.entry_mut(entity).ok().map(|mut e| {
        e.get_component_mut::<HierarchyComponent>()
            .map(|h| h.parent = Some(new_root))
    });
}

pub(crate) fn collect_hierarchy_root_entities(world: &legion::World) -> Vec<Entity> {
    use legion::query::IntoQuery;

    let mut query = <(Entity, &HierarchyComponent)>::query();
    query
        .iter(world)
        .filter(|(e, _)| is_root(**e, world))
        .map(|(e, _)| *e)
        .collect()
}

pub(crate) fn has_hierarchy(entity: Entity, world: &legion::World) -> bool {
    let Ok(entry) = world.entry_ref(entity) else {
        return false;
    };
    entry.get_component::<HierarchyComponent>().is_ok()
}

pub(crate) fn remove_entity(
    asset_mgr: &mut AssetManager,
    entity: Entity,
    world: &mut legion::World,
) {
    if has_hierarchy(entity, world) {
        remove_meshes_from_all(asset_mgr, entity, world);
    } else {
        // is light
        world.remove(entity);
    }
}

fn remove_meshes_from_all(asset_mgr: &mut AssetManager, entity: Entity, world: &mut legion::World) {
    let entities = collect_mesh_entities_from_root(entity, world);
    let mesh_ids = collect_mesh_asset_ids_from_entity(world, &entities);

    // remove entity from world
    for e in entities {
        world.remove(e);
    }

    // remove mesh from asset
    // asset will remove dependency : (material, textures)
    // TODO: check if mesh is shared by others before removing
    for mesh_id in mesh_ids {
        asset_mgr.remove(mesh_id);
    }
}

type GlobalAssetIDCollection = Vec<GlobalAssetId>;

fn collect_mesh_asset_ids_from_entity(
    world: &legion::World,
    entities: &Vec<Entity>,
) -> GlobalAssetIDCollection {
    let mut ids = vec![];

    for e in entities.clone() {
        if let Ok(entry) = world.entry_ref(e) {
            if let Ok(mesh) = entry.get_component::<MeshComponent>() {
                ids.push(mesh.handle);
            }
        }
    }
    ids
}

fn is_root(entity: Entity, world: &legion::World) -> bool {
    let Ok(entry) = world.entry_ref(entity) else {
        return false;
    };

    let Ok(hierarchy) = entry.get_component::<HierarchyComponent>() else {
        return false;
    };

    hierarchy.parent.is_none()
}

fn collect_mesh_entities_from_root(entity: Entity, world: &mut legion::World) -> Vec<Entity> {
    let mut entities = Vec::new();
    if !is_root(entity, world) {
        warn!("{:?} Not Root: Remove abort", entity);
        return entities;
    }

    fn collect_subtree(world: &legion::World, root: Entity, out: &mut Vec<Entity>) {
        out.push(root);

        if let Ok(entry) = world.entry_ref(root) {
            if let Ok(h) = entry.get_component::<HierarchyComponent>() {
                for &child in &h.children {
                    collect_subtree(world, child, out);
                }
            }
        }
    }

    collect_subtree(world, entity, &mut entities);

    entities
}
