pub mod bounding_box;
pub mod light;
pub mod mesh;

use legion::{Entity, EntityStore};
use std::mem;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct EntityId(pub u64);

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

use std::hash::{Hash, Hasher};

use crate::HierarchyComponent;
pub trait EntityHash {
    /// Restituisce un hash `u64` deterministico
    fn entity_hash(&self) -> u64;
}

impl EntityHash for Entity {
    fn entity_hash(&self) -> u64 {
        let mut hasher = std::hash::DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}


fn collect_subtree(world: &legion::world::SubWorld, root: Entity, out: &mut Vec<Entity>) {
    out.push(root);

    if let Ok(entry) = world.entry_ref(root) {
        if let Ok(h) = entry.get_component::<HierarchyComponent>() {
            for &child in &h.children {
                collect_subtree(world, child, out);
            }
        }
    }
}

pub fn remove_from_root(entity: Entity, world: &mut legion::world::SubWorld, cmd: &mut legion::systems::CommandBuffer) {
    let mut to_delete = Vec::new();
    collect_subtree(world, entity, &mut to_delete);
    for e in to_delete.into_iter().rev() {
        cmd.remove(e);
    }
}

pub fn add_parent(entity: Entity, world: &mut legion::world::SubWorld, cmd: &mut legion::systems::CommandBuffer) {
    // if not root node do nothing
    if let Ok(entry) = world.entry_mut(entity) {
        if let Ok(hierarchy) = entry.get_component::<HierarchyComponent>() {
            if hierarchy.parent.is_some() {
                return;
            }
        }
    }
    // Add parent node and set entity as child
    let new_root = {
        cmd.push((
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
    if let Ok(mut entry) = world.entry_mut(entity) {
        let hierarchy = entry
            .get_component_mut::<crate::HierarchyComponent>()
            .unwrap();
        hierarchy.parent = Some(new_root);
    };
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
