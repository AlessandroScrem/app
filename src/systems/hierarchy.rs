use cgmath::Matrix4;
use legion::{
    systems::CommandBuffer,
    world::SubWorld,
    *,
};

use crate::{
    HierarchyComponent, MeshComponent, TransformComponent, entities::EntityRawU64,
    renderer::uniform::ModelUniform,
};

#[system]
#[read_component(TransformComponent)]
#[read_component(HierarchyComponent)]
pub fn hieararchy(world: &SubWorld, commands: &mut CommandBuffer) {
    let mut query = <(Entity, Read<HierarchyComponent>, Read<TransformComponent>)>::query();

    // Entities with a `HierarchyComponent` and NOT a `Parent` (ie those that are
    // roots of a hierarchy).
    for (entity, hirarchy, transform) in query.iter(world).filter(|(_e, h, _t)| h.parent.is_none())
    {
        // Calcolo della matrice globale
        let local_matrix = transform.compute_model_matrix();

        // Aggiorna uniform
        let mut updated_uniform = ModelUniform::new(local_matrix);
        updated_uniform.entity_id = entity.as_raw_u64();

        // Aggiorna o sostituisce il componente
        commands.add_component(*entity, updated_uniform);

        // Propaga ai figli
        for child in hirarchy.children.iter() {
            propagate_recursive(local_matrix, world, *child, commands);
        }
    }
}

fn propagate_recursive(
    parent_matrix: Matrix4<f32>,
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

    // Aggiorna uniform
    let mut updated_uniform = ModelUniform::new(local_matrix);
    updated_uniform.entity_id = entity.as_raw_u64();
    commands.add_component(entity, updated_uniform);

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

#[system]
#[read_component(ModelUniform)]
#[read_component(MeshComponent)]
pub fn hierarchy_update_uniforms(world: &mut SubWorld, #[resource] queue: &wgpu::Queue) {
    let mut uniforms_query = <(Entity, &MeshComponent, &ModelUniform)>::query();
    for (_entity, mesh, model_uniform) in uniforms_query.iter(world) {
        queue.write_buffer(
            &mesh.data.model_uniform_buffer,
            0,
            bytemuck::bytes_of(model_uniform),
        );
    }
}
