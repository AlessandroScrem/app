
use legion::*;

use crate::LightComponent;

/// A function to help create a light entity.
pub fn create(world: &mut World, _resources: &Resources) {

    let light = LightComponent {name: "Directional".to_string(), ..Default::default()};

    world.push((light,));

}
