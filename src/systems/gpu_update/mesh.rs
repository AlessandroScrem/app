use crate::{
    GlobalModelComponent, MeshComponent,
    assets::{material_manager::MaterialManager, mesh_manager::MeshManager},
    entities::EntityRawU64,
    renderer::uniform::{MaterialUniform, ModelUniform},
};

use legion::{world::SubWorld, *};

#[system(for_each)]
#[filter(maybe_changed::<MeshComponent>())]
pub fn update_material_system_to_gpu(
    mesh: &MeshComponent,
    #[resource] queue: &wgpu::Queue,
    #[resource] material_manager: &MaterialManager,
) {
    let material = material_manager.get(&mesh.mat_handle);
    let buffer = &material.uniform_buffer;
    let updated_uniforms = MaterialUniform::from(&material.material_pbr);
    queue.write_buffer(buffer, 0, bytemuck::bytes_of(&updated_uniforms));
}

#[system]
#[read_component(GlobalModelComponent)]
#[read_component(MeshComponent)]
pub fn update_model_uniforms_to_gpu(
    world: &mut SubWorld,
    #[resource] queue: &wgpu::Queue,
    #[resource] mesh_manager: &MeshManager,
) {
    let mut uniforms_query = <(Entity, &MeshComponent, &GlobalModelComponent)>::query();

    for (entity, mesh, global_model) in uniforms_query.iter(world) {
        let mut model_uniform = ModelUniform::new(global_model.mat);
        model_uniform.entity_id = entity.as_raw_u64();
        queue.write_buffer(
            &mesh_manager.get_model_uniform(mesh.handle),
            0,
            bytemuck::bytes_of(&model_uniform),
        );
    }
}
