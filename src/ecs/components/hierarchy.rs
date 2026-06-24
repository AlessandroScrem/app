use super::components::*;
use crate::assets::asset_manager::AssetManager;
use crate::assets::{MeshAsset, MaterialAsset};
use crate::assets::material_desc::MaterialTextureSlot;
use crate::prelude::warn;

use legion::*;

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

pub(crate) fn remove_entity_from_all(
    asset_mgr: &mut AssetManager,
    entity: Entity,
    world: &mut legion::World,
) {
    let entities = collect_entity_from_root(entity, world);
    let IDCollection {
        mesh_ids,
        material_ids,
        texture_ids,
    } = collect_asset_ids_from_entity(world, asset_mgr, &entities);

    // remove entity from world
    for e in entities {
        world.remove(e);
    }

    // remove mesh from asset
    // TODO: check if mesh is shared by others before removing
    for mesh_id in mesh_ids {
        asset_mgr.remove(mesh_id);
    }

    // remove material from asset
    // remove also textures from slot
    for mat_id in material_ids {
        asset_mgr.remove(mat_id);
    }

    // remove texture from asset
    // texture slot are removed from materials.remove()
    for tex_id in texture_ids {
        asset_mgr.remove(tex_id);
    }
}

struct IDCollection {
    mesh_ids: Vec<crate::assets::MeshId>,
    material_ids: Vec<crate::assets::MaterialId>,
    texture_ids: Vec<crate::assets::TextureId>,
}
fn collect_asset_ids_from_entity(
    world: &legion::World,
    asset_mgr: &AssetManager,
    entities: &Vec<Entity>,
) -> IDCollection {
    let mut mesh_ids = vec![];
    let mut material_ids = vec![];
    let mut texture_ids = vec![];

    for e in entities.clone() {
        if let Ok(entry) = world.entry_ref(e) {
            if let Ok(mesh) = entry.get_component::<MeshComponent>() {
                mesh_ids.push(mesh.handle);
                if let Some(mesh_asset) = asset_mgr.get::<MeshAsset>(mesh.handle) {
                    for submesh in mesh_asset.desc.submeshes.iter() {
                        let mat_id = submesh.material;
                        material_ids.push(mat_id);
                        if let Some(mat_asset) = asset_mgr.get::<MaterialAsset>(mat_id) {
                            for slot in MaterialTextureSlot::ALL {
                                if let Some(tex_id) = mat_asset.desc.texture(slot) {
                                    texture_ids.push(tex_id);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    IDCollection {
        mesh_ids,
        material_ids,
        texture_ids,
    }
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

fn collect_entity_from_root(entity: Entity, world: &mut legion::World) -> Vec<Entity> {
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

