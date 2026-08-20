use legion::{Entity, EntityStore, World};

use crate::{
    EntityRawU64, Globals,
    assets::MeshId,
    ecs::components::{
        BoundingBoxComponent, GlobalModelComponent, Hidden, HierarchyComponent, LightComponent,
        MeshComponent,
    },
    math::Mat4,
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

pub struct BboxRenderObject {
    #[allow(dead_code)]
    pub entity_id: u64,
    pub bbox: BoundingBoxComponent,
    pub transform: Mat4,
}

#[derive(Default)]
pub struct RenderObjects {
    pub meshes: Vec<MeshRenderObject>,
    pub lights: Vec<LightRenderObject>,
    pub bboxes: Vec<BboxRenderObject>,
}

impl RenderObjects {
    pub fn build(world: &World, globals: &Globals) -> Self {
        let meshes = extract_meshes(world);
        let lights = extract_lights(world);
        let bboxes = extract_bbox(world, globals.bbox_enable);

        Self {
            meshes,
            bboxes,
            lights,
        }
    }
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
        return is_hidden(world, parent);
    }

    false
}

fn extract_meshes(world: &World) -> Vec<MeshRenderObject> {
    use legion::IntoQuery;
    let mut query = <(Entity, &MeshComponent, &GlobalModelComponent)>::query();

    let mut meshes = Vec::new();
    for (entity, mesh, transform) in query.iter(world) {
        if is_hidden(world, *entity) {
            continue;
        }
        meshes.push(MeshRenderObject {
            entity_id: entity.as_raw_u64(),
            mesh: mesh.handle,
            transform: transform.mat,
        });
    }
    meshes
}

fn extract_lights(world: &World) -> Vec<LightRenderObject> {
    use legion::IntoQuery;
    let mut query = <(Entity, &LightComponent)>::query();

    let mut lights = Vec::new();
    for (entity, light) in query.iter(world) {
        if !light.enabled {
            continue;
        }

        lights.push(LightRenderObject {
            entity_id: entity.as_raw_u64(),
            light: light.clone(),
        });
    }
    lights
}

fn extract_bbox(world: &World, bbox_enable: bool) -> Vec<BboxRenderObject> {
    if !bbox_enable {
        return Vec::new();
    }

    use legion::IntoQuery;
    let mut query = <(Entity, &BoundingBoxComponent, &GlobalModelComponent)>::query();

    let mut bboxes = Vec::new();
    query.for_each(world, |(entity, bbox, transform)| {
        bboxes.push(BboxRenderObject {
            entity_id: entity.as_raw_u64(),
            bbox: bbox.clone(),
            transform: transform.mat,
        });
    });
    bboxes
}
