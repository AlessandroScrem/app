use legion::*;

use crate::{LightComponent, TagComponent};
use legion::Entity;
use legion::world::World;
use std::hash::{Hash, Hasher};

use std::collections::hash_map::DefaultHasher;

pub fn entity_to_u32(entity: Entity) -> u32 {
    let mut hasher = DefaultHasher::new();
    entity.hash(&mut hasher);
    (hasher.finish() & 0xFFFF_FFFF) as u32 // prendo solo 32 bit
}

/// A function to help create a light entity.
pub fn create(world: &mut World, _resources: &Resources) {
    let mut light = LightComponent::default();
    light.data.position = [0.0, 2.0, 0.0];

    let entity: Entity = world.push((
        TagComponent {
            name: "Directional1".to_string(),
        },
        light,
    ));

    if let Some(mut entry) = world.entry(entity) {
        let comp = entry.get_component_mut::<LightComponent>().unwrap();
        comp.data.entity_id =  entity_to_u32(entity) as i32;
    }
}
