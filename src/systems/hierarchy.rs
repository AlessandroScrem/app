use cgmath::Matrix4;
use legion::{world::SubWorld, *};

use crate::{
    HierarchyComponent, MeshComponent, TransformComponent, entities::EntityRawU64,
    renderer::uniform::ModelUniform,
};

use std::collections::HashMap;

#[system]
#[read_component(TransformComponent)]
#[read_component(HierarchyComponent)]
pub fn compute_global_transforms(
    world: &mut SubWorld,
    #[resource] transforms: &mut HashMap<Entity, Matrix4<f32>>,
) {
    let entities: Vec<(Entity, TransformComponent, HierarchyComponent)> = 
        <(Entity, &TransformComponent, &HierarchyComponent)>::query()
        .iter(world)
        .map(|(e, t, h)| (*e, t.clone(), h.clone()))
        .collect();

    transforms.clear();
  
    fn compute_child_transforms_recurse(
        parent: &Entity,
        parent_matrix: &Matrix4<f32>,
        entities: &[(Entity, TransformComponent, HierarchyComponent)],
        model_matrices: &mut HashMap<Entity, Matrix4<f32>>,
    ) {
        model_matrices.insert(*parent, *parent_matrix);
    
        for (entity, transform, hierarchy) in entities {
            if hierarchy.parent == Some(*parent) {
                let local = transform.compute_model_matrix();
                let global = *parent_matrix * local;
                compute_child_transforms_recurse(entity, &global, entities, model_matrices);
            }
        }
    }

    for (entity, transform, hierarchy) in &entities {
        if hierarchy.parent.is_none() {
            let model = transform.compute_model_matrix();
            compute_child_transforms_recurse(entity, &model, &entities, transforms);
        }
    }
}

#[system]
#[write_component(ModelUniform)]
#[read_component(MeshComponent)]
pub fn update_model_uniforms(
    world: &mut SubWorld,
    #[resource] queue: &wgpu::Queue,
    #[resource] transforms: &HashMap<Entity, Matrix4<f32>>,
) {
    let mut uniforms_query = <(Entity, &MeshComponent, &mut ModelUniform)>::query();
    for (entity, mesh, model_uniform) in uniforms_query.iter_mut(world) {
        if let Some(model) = transforms.get(&entity) {
            let mut updated = ModelUniform::new(*model);
            updated.entity_id = entity.as_raw_u64();
            *model_uniform = updated;

            queue.write_buffer(
                &mesh.data.model_uniform_buffer,
                0,
                bytemuck::bytes_of(&updated),
            );
        }
    }
}
