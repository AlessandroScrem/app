use legion::*;

use crate::entities::EntityRawU64;
use crate::{LightComponent, TagComponent};
use legion::Entity;
use legion::world::World;

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
        comp.data.entity_id = entity.as_raw_u64();
    }
}


