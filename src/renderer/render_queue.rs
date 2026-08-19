use legion::{Entity, EntityStore, World};

use crate::{
    EntityRawU64, Globals,
    assets::{LinesVertexData, MeshId},
    ecs::components::{
        BoundingBoxComponent, GlobalModelComponent, Hidden, HierarchyComponent, LightComponent,
        MeshComponent,
    },
    math::Mat4,
    renderer::{
        line_builder::{AxisAlignedBoundingBox, LineDrawable, ObjectOrientedBoundingBox},
        uniform,
    },
};


// ------------------------------------
// ------------------------------------
pub struct MeshRenderObject {
    pub entity_id: u64,
    pub mesh: MeshId,
    pub transform: Mat4,
}

pub struct LightRenderObject {
    pub entity_id: u64,
    pub light: LightComponent,
}


#[derive(Default)]
pub struct RenderQueue {
    pub mesh: Vec<MeshRenderObject>,
    pub lights: Vec<LightRenderObject>,
    pub lines: Vec<LinesVertexData>,
}

impl RenderQueue {
    fn mesh(&mut self, mesh: MeshId, transform: Mat4, entity: Entity) {
        self.mesh.push(MeshRenderObject {
            entity_id: entity.as_raw_u64(),
            mesh,
            transform,
        });
    }

    fn light(&mut self, entity: Entity, light: &LightComponent) {
        if !light.enabled {
            return;
        }

        self.lights.push(LightRenderObject {
            entity_id: entity.as_raw_u64(),
            light: light.clone(),
        });
    }

    pub fn build(&mut self, world: &World, globals: &Globals) {
        extract_meshes(world, self);
        extract_lights(world, self);
        extract_bbox(world, globals, self);
        extract_light_frustums(world, self);
    }

    fn is_hidden(world: &World, entity: Entity) -> bool {
        let Ok(entry) = world.entry_ref(entity) else {
            return false;
        };
        // check if has Hidden component
        if entry.get_component::<Hidden>().is_ok() {
            return true;
        }

        let Ok(hierarchy) = entry.get_component::<HierarchyComponent>() else {
            return false;
        };

        // recurse to parent
        if let Some(parent) = hierarchy.parent {
            return Self::is_hidden(world, parent);
        }

        false
    }
}

fn extract_meshes(world: &World, queue: &mut RenderQueue) {
    use legion::IntoQuery;
    let mut query = <(Entity, &MeshComponent, &GlobalModelComponent)>::query();
    for (entity, mesh, transform) in query.iter(world) {
        if RenderQueue::is_hidden(world, *entity) {
            continue;
        }
        queue.mesh(mesh.handle, transform.mat, *entity);
    }
}

fn extract_lights(world: &World, queue: &mut RenderQueue) {
    use legion::IntoQuery;
    let mut query = <(Entity, &LightComponent)>::query();

    for (entiy, light) in query.iter(world) {
        queue.light(*entiy, light);
    }
}

fn extract_bbox(world: &World, globals: &Globals, queue: &mut RenderQueue) {
    if !globals.bbox_enable {
        return;
    }

    let axis_aligned = globals.bbox_axis_aligned;

    use legion::IntoQuery;
    let mut query = <(&BoundingBoxComponent, &GlobalModelComponent)>::query();

    query.for_each(world, |(bbox, transform)| {
        if axis_aligned {
            AxisAlignedBoundingBox {
                bbox: &bbox.global_bounding_box,
            }
            .emit(&mut queue.lines);
        } else {
            ObjectOrientedBoundingBox {
                bbox: &bbox.bounding_box,
                transform: &transform.mat,
            }
            .emit(&mut queue.lines);
        }
    });
}

fn extract_light_frustums(world: &World, queue: &mut RenderQueue) {
    use legion::IntoQuery;
    let mut query = <&LightComponent>::query();

    for light in query
        .iter(world)
        .filter(|l| l.frustum)
        .take(uniform::MAX_LIGHTS)
    {
        light.emit(&mut queue.lines);
    }
}
