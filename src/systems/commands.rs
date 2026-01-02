use legion::{systems::CommandBuffer, world::SubWorld, *};

use crate::{
    entities::*,
    DomainEvent, DomainEvents, HierarchyComponent, LightComponent, MeshComponent, TagComponent,
    TransformComponent,
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
    #[resource] events: &mut DomainEvents,
) {
    while let Some(event) = events.queue.pop_front() {
        match event {
            DomainEvent::RemoveEntity(entity) => {
                remove_from_root(entity, world, cmd);
            }
            DomainEvent::LoadGltf(path) => {
                // create_from_gltf(path, world, &mut self.resources);
            }
            DomainEvent::AddParent(entity) => {
                add_parent(entity, world, cmd);
            }
            _ => {}
        }
    }
}
