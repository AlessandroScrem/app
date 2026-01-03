use legion::{systems::CommandBuffer, world::SubWorld, *};

use crate::{
    DomainEvent, DomainEvents, GlobalModelComponent, HierarchyComponent, LightComponent, MeshComponent, TagComponent, TransformComponent, assets::{
        material_manager::MaterialManager, mesh_manager::MeshManager,
        texture_manager::TextureManager,
    }, entities::*, picking::PickObject, renderer::GpuManager
};

use crate::assets::mesh::*;

#[system]
#[write_component(TagComponent)]
#[write_component(MeshComponent)]
#[write_component(TransformComponent)]
#[write_component(HierarchyComponent)]
#[write_component(GlobalModelComponent)]
#[write_component(LightComponent)]
pub fn apply_commands(
    world: &mut SubWorld,
    cmd: &mut CommandBuffer,
    #[resource] device: &wgpu::Device,
    #[resource] mat_mgr: &mut MaterialManager,
    #[resource] mesh_mgr: &mut MeshManager,
    #[resource] tex_mgr: &mut TextureManager,
    #[resource] gpu_mgr: &GpuManager,
    #[resource] events: &mut DomainEvents,
    #[resource] pick_object: &mut PickObject,
) {
    let mut gpu = GpuDevice {
        device,
        gpu_mgr,
        mat_mgr,
        mesh_mgr,
        tex_mgr
    };

    while let Some(event) = events.queue.pop_front() {
        match event {
            DomainEvent::RemoveEntity(entity) => {
                pick_object.selected = None;
                pick_object.hovered = None;
                remove_from_root(entity, world, cmd);
            }
            DomainEvent::LoadGltf(path) => {
                let loaded = load(path).unwrap();
                let gpu_scene = upload_scene_to_gpu(&loaded, &mut gpu);
                spawn_scene(cmd, &loaded, &gpu_scene);  
            },
            DomainEvent::AddParent(entity) => {
                add_parent(entity, world, cmd);
            }
        }
    }
}
