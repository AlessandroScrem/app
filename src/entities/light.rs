
use legion::*;

use crate::{LightComponent, TagComponent};

/// A function to help create a light entity.
pub fn create(world: &mut World, _resources: &Resources) {

    world.push((TagComponent {name: "Directional1".to_string()}, LightComponent::default()));

}
