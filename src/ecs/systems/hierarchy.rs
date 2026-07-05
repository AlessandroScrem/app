use legion::{systems::CommandBuffer, world::SubWorld, *};

use crate::ecs::components::{GlobalModelComponent, HierarchyComponent, TransformComponent};
use crate::math::*;

#[system]
#[read_component(TransformComponent)]
#[read_component(HierarchyComponent)]
pub fn update_hieararchy(world: &SubWorld, commands: &mut CommandBuffer) {
    let mut query = <(Entity, Read<HierarchyComponent>, Read<TransformComponent>)>::query();

    // Entities with a `HierarchyComponent` and NOT a `Parent`
    // (roots of a hierarchy)
    for (entity, hirarchy, transform) in query.iter(world).filter(|(_e, h, _t)| h.parent.is_none())
    {
        // Calcolo della matrice globale
        let local_matrix = transform.compute_model_matrix();
        let global_model = GlobalModelComponent { mat: local_matrix };

        // Aggiorna o sostituisce il componente
        commands.add_component(*entity, global_model);

        // Propaga ai figli
        for child in hirarchy.children.iter() {
            propagate_recursive(local_matrix, world, *child, commands);
        }
    }
}

fn propagate_recursive(
    parent_matrix: Mat4,
    world: &SubWorld,
    entity: Entity,
    commands: &mut CommandBuffer,
) {
    // Ottieni la matrice locale
    let local_matrix = {
        let entry = match world.entry_ref(entity) {
            Ok(e) => e,
            Err(_) => {
                log::warn!("Entity {:?} not found in world", entity);
                return;
            }
        };

        if let Ok(transform) = entry.get_component::<TransformComponent>() {
            transform.compute_model_matrix()
        } else {
            log::warn!(
                "Entity {:?} is a child in the hierarchy but does not have a TransformComponent",
                entity
            );
            return;
        }
    };

    // Calcolo della matrice globale
    let local_matrix = parent_matrix * local_matrix;

    // Aggiorna o sostituisce il componente
    let global_model = GlobalModelComponent { mat: local_matrix };
    commands.add_component(entity, global_model);

    // Propaga ai figli
    let children = {
        let entry = match world.entry_ref(entity) {
            Ok(e) => e,
            Err(_) => return,
        };

        if let Ok(hierarchy) = entry.get_component::<HierarchyComponent>() {
            hierarchy.children.clone()
        } else {
            return;
        }
    };

    for child in children {
        propagate_recursive(local_matrix, world, child, commands);
    }
}
