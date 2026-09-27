use legion::*;

use crate::ecs::components::{
    GlobalModelComponent, Hidden, HierarchyComponent, TransformComponent,
};

use super::components::{LightComponent, TagComponent};
use legion::world::World;

/// A function to help create a light entity.
pub fn create(world: &mut World) -> Entity {
    world.push((
        TagComponent {
            name: "Directional".to_string(),
        },
        TransformComponent{position: [3.0, 20.0, 10.0], ..Default::default()},
        HierarchyComponent::default(),
        GlobalModelComponent::default(),
        LightComponent::default(),
    ))
}

/// A function to help create a light entity.
pub fn add_light(world: &mut World, light: LightComponent, name: TagComponent, transform: TransformComponent) -> Entity {
    world.push((
        name,
        transform,
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
