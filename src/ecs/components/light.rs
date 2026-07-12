use legion::*;

use super::components::{LightComponent, TagComponent};
use legion::world::World;

/// A function to help create a light entity.
pub fn create(world: &mut World) ->Entity {
    let mut light = LightComponent::default();
    light.update_position([3.0, 20.0, 10.0]);

    world.push((
        TagComponent {
            name: "Directional".to_string(),
        },
        light,
    ))
}

pub fn enable_all_lights(enable: bool, world: &mut legion::World) {
    use legion::query::IntoQuery;

    let mut query = <&mut LightComponent>::query();

    for light in query.iter_mut(world) {
        light.enabled = enable;
    }
}