pub(crate) mod bounding_box_impl;
pub(crate) mod components;
pub(crate) mod light;

pub(crate) use components::*;

use legion::{Entity, EntityStore};
use log::warn;
use std::mem;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct EntityId(pub(crate) u64);

pub trait EntityRawU64 {
    fn as_raw_u64(&self) -> u64;
    fn from_raw_u64(raw: u64) -> Self;
}

impl EntityRawU64 for Entity {
    fn as_raw_u64(&self) -> u64 {
        unsafe {
            let raw64: u64 = mem::transmute(*self);
            raw64
        }
    }

    fn from_raw_u64(raw: u64) -> Self {
        unsafe {
            let raw64: u64 = raw as u64;
            mem::transmute::<u64, Entity>(raw64)
        }
    }
}

impl From<Entity> for EntityId {
    fn from(e: Entity) -> Self {
        EntityId(e.as_raw_u64())
    }
}

impl From<EntityId> for Entity {
    fn from(id: EntityId) -> Self {
        Entity::from_raw_u64(id.0)
    }
}

use crate::{
    AssetManager,
    assets::{MaterialTextureSlot, gltf_loader::LoadedScene},
};

// use std::hash::{Hash, Hasher};
// pub(crate) trait EntityHash {
//     /// Restituisce un hash `u64` deterministico
//     fn entity_hash(&self) -> u64;
// }

// impl EntityHash for Entity {
//     fn entity_hash(&self) -> u64 {
//         let mut hasher = std::hash::DefaultHasher::new();
//         self.hash(&mut hasher);
//         hasher.finish()
//     }
// }

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

fn is_root(entity: Entity, world: &legion::World) -> bool {
    let Ok(entry) = world.entry_ref(entity) else {
        return false;
    };

    let Ok(hierarchy) = entry.get_component::<HierarchyComponent>() else {
        return false;
    };

    hierarchy.parent.is_none()
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
            crate::TagComponent {
                name: "New Node".into(),
            },
            crate::HierarchyComponent {
                parent: None,
                children: vec![entity],
            },
            crate::GlobalModelComponent::default(),
            crate::TransformComponent::default(),
        ))
    };

    // Register new node as root
    world.entry_mut(entity).ok().map(|mut e| {
        e.get_component_mut::<HierarchyComponent>()
            .map(|h| h.parent = Some(new_root))
    });
}

struct IDCollection {
    mesh_ids: Vec<crate::assets::MeshId>,
    material_ids: Vec<crate::assets::MaterialId>,
    texture_ids: Vec<crate::assets::TextureId>,
}
fn collect_asset_ids_from_entity(
    world: &legion::World,
    asset_mgr: &mut crate::AssetManager,
    entities: &Vec<Entity>,
) -> IDCollection {
    let mut mesh_ids = vec![];
    let mut material_ids = vec![];
    let mut texture_ids = vec![];

    for e in entities.clone() {
        if let Ok(entry) = world.entry_ref(e) {
            if let Ok(mesh) = entry.get_component::<MeshComponent>() {
                mesh_ids.push(mesh.handle);
                if let Some(mesh_desc) = asset_mgr.meshes.get(mesh.handle) {
                    for submesh in mesh_desc.submeshes.iter() {
                        let mat_id = submesh.material;
                        material_ids.push(mat_id);
                        if let Some(mat_desc) = asset_mgr.materials.get_desc(mat_id) {
                            for slot in MaterialTextureSlot::ALL {
                                if let Some(tex_id) = mat_desc.texture(slot) {
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

pub(crate) fn remove_entity_from_all(
    asset_mgr: &mut crate::AssetManager,
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
        asset_mgr.meshes.remove(mesh_id);
    }

    // remove material from asset
    // remove also textures from slot
    for mat_id in material_ids {
        asset_mgr.materials.remove(mat_id, &mut asset_mgr.textures);
    }

    // remove texture from asset
    // texture slot are removed from materials.remove()
    for tex_id in texture_ids {
        asset_mgr.textures.remove(tex_id);
    }
}

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
                if let Some(mesh) = asset_mgr.meshes.get(*mesh_id) {
                    let bbox = &mesh.bounds;
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

#[cfg(test)]
mod tests {
    use super::*;
    use legion::*;

    #[test]
    fn test_entity_to_raw_u64_and_back() {
        let mut world = World::default();
        let e = world.push((10.0,));

        let e_id = EntityId::from(e);
        let result: Entity = e_id.into();

        assert_eq!(e, result);
    }

    #[test]
    // Ricostruzione come nello shader: high << 32 | low
    // Da usare nello shader per ricostruire entity_id (u64) da vec2<u32>
    fn test_reconstruct_u64_from_u32() {
        use std::u32;
        // Valore u64 più grande di u32::MAX
        let original: u64 = u32::MAX as u64 + 48; // 4.294.967.343

        // Split in low/high 32 bit
        let low: u32 = original as u32; // parte bassa
        let high: u32 = (original >> 32) as u32; // parte alta

        let reconstructed: u64 = (high as u64) << 32 | (low as u64);

        // Verifica
        assert_eq!(original, reconstructed);

        // Stampa per controllo
        println!("original = {}", original);
        println!("low = {}, high = {}", low, high);
        println!("reconstructed = {}", reconstructed);
    }
}
