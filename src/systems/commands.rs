use legion::{systems::CommandBuffer, world::SubWorld, *};

use crate::{
    DomainEvent, DomainEvents, HierarchyComponent, LightComponent, MeshComponent, TagComponent,
    TransformComponent,
    assets::{
        material_manager::MaterialManager, mesh_manager::MeshManager,
        texture_manager::TextureManager,
    },
    entities::*,
    renderer::GpuManager,
};

#[system]
#[write_component(TagComponent)]
#[write_component(MeshComponent)]
#[write_component(TransformComponent)]
#[write_component(HierarchyComponent)]
#[write_component(LightComponent)]
pub fn apply_commands(
    world: &mut SubWorld,
    cmd: &mut CommandBuffer,
    #[resource] device: &wgpu::Device,
    #[resource] mesh_manager: &mut MeshManager,
    #[resource] mat_manager: &mut MaterialManager,
    #[resource] texture_manager: &mut TextureManager,
    #[resource] gpu_manager: &GpuManager,
    #[resource] events: &mut DomainEvents,
) {
    while let Some(event) = events.queue.pop_front() {
        match event {
            DomainEvent::RemoveEntity(entity) => {
                remove_from_root(entity, world, cmd);
            }
            DomainEvent::LoadGltf(path) => mesh::create_from_gltf(
                path,
                cmd,
                device,
                mesh_manager,
                mat_manager,
                texture_manager,
                gpu_manager,
            ),
            DomainEvent::AddParent(entity) => {
                add_parent(entity, world, cmd);
            }
        }
    }
}
