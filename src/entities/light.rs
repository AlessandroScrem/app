use legion::*;

use crate::{LightComponent, TagComponent};

/// A function to help create a light entity.
pub fn create(world: &mut World, _resources: &Resources) {
    let mut light = LightComponent::default();
    light.data.position = [0.0, 2.0, 0.0];

    world.push((
        TagComponent {
            name: "Directional1".to_string(),
        },
        light,
    ));
}
