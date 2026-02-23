use legion::*;

use crate::{LightComponent, TagComponent};
use legion::world::World;

/// A function to help create a light entity.
pub(crate) fn create(world: &mut World, _resources: &Resources) {
    let mut light = LightComponent::default();
    light.data.position = [3.0, 20.0, 10.0];

    world.push((
        TagComponent {
            name: "Directional1".to_string(),
        },
        light,
    ));
}


