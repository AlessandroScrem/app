use cgmath::{ElementWise, Rotation};
use legion::{systems::CommandBuffer, world::SubWorld, *};

use crate::ecs::components::{GlobalModelComponent, HierarchyComponent, TransformComponent};
use crate::math::*;

#[derive(Clone, Copy)]
struct GlobalTransform {
    position: Vec3,
    rotation: Quat,
    scale: Vec3,
}

impl GlobalTransform {
    fn from_local(transform: &TransformComponent) -> Self {
        Self {
            position: Vec3::from(transform.position),
            rotation: quat_from_euler(&transform.rotation),
            scale: Vec3::from(transform.scale),
        }
    }

    fn combine(self, local: &TransformComponent) -> Self {
        let local_position = Vec3::from(local.position);
        let local_rotation = quat_from_euler(&local.rotation);
        let local_scale = Vec3::from(local.scale);

        Self {
            position: self.position
                + self.rotation.rotate_vector(local_position.mul_element_wise(self.scale)),
            rotation: self.rotation * local_rotation,
            scale: self.scale.mul_element_wise(local_scale),
        }
    }

    fn matrix(self) -> Mat4 {
        Mat4::from_translation(self.position)
            * Mat4::from(self.rotation)
            * Mat4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
    }
}

fn quat_from_euler(rotation: &[f32; 3]) -> Quat {
    Quat::from(Euler::new(Rad(rotation[0]), Rad(rotation[1]), Rad(rotation[2])))
}

#[system]
#[read_component(TransformComponent)]
#[read_component(HierarchyComponent)]
pub fn update_hieararchy(world: &SubWorld, commands: &mut CommandBuffer) {
    let mut query = <(Entity, Read<HierarchyComponent>, Read<TransformComponent>)>::query();

    for (entity, hierarchy, transform) in query.iter(world).filter(|(_, h, _)| h.parent.is_none()) {
        let global = GlobalTransform::from_local(transform);
        commands.add_component(*entity, GlobalModelComponent { mat: global.matrix() });

        for child in &hierarchy.children {
            propagate_recursive(global, world, *child, commands);
        }
    }
}

fn propagate_recursive(
    parent_global: GlobalTransform,
    world: &SubWorld,
    entity: Entity,
    commands: &mut CommandBuffer,
) {
    let local_transform = {
        let entry = match world.entry_ref(entity) {
            Ok(e) => e,
            Err(_) => {
                log::warn!("Entity {:?} not found in world", entity);
                return;
            }
        };

        match entry.get_component::<TransformComponent>() {
            Ok(transform) => transform.clone(),
            Err(_) => {
                log::warn!(
                    "Entity {:?} is a child in the hierarchy but does not have a TransformComponent",
                    entity
                );
                return;
            }
        }
    };

    let global = parent_global.combine(&local_transform);
    commands.add_component(entity, GlobalModelComponent { mat: global.matrix() });

    let children = {
        let entry = match world.entry_ref(entity) {
            Ok(e) => e,
            Err(_) => return,
        };

        match entry.get_component::<HierarchyComponent>() {
            Ok(hierarchy) => hierarchy.children.clone(),
            Err(_) => return,
        }
    };

    for child in children {
        propagate_recursive(global, world, child, commands);
    }
}
