use legion::*;

use crate::ecs::components::{
    GlobalModelComponent, Hidden, HierarchyComponent, TransformComponent,
};

use super::components::{LightComponent, TagComponent};
use legion::world::World;

/// A function to help create a light entity.
pub fn create(world: &mut World) -> Entity {
    let mut light = LightComponent::default();
    light.update_position([3.0, 20.0, 10.0]);

    world.push((
        TagComponent {
            name: "Directional".to_string(),
        },
        TransformComponent::default(),
        HierarchyComponent::default(),
        GlobalModelComponent::default(),
        light,
    ))
}

/// A function to help create a light entity.
pub fn add_light(world: &mut World, light: LightComponent, name: TagComponent) -> Entity {
    world.push((
        name,
        TransformComponent::default(),
        HierarchyComponent::default(),
        GlobalModelComponent::default(),
        light,
    ))
}

pub fn enable_all_lights(enable: bool, world: &mut legion::World) {
    use legion::query::IntoQuery;

    let mut query = <(Entity, &LightComponent)>::query();
    let entities: Vec<Entity> = query.iter(world).map(|(entity, _)| *entity).collect();

    for entity in entities {
        if let Some(mut e) = world.entry(entity) {
            if enable {
                e.remove_component::<Hidden>();
            } else {
                e.add_component(Hidden);
            }
        }
    }
}
