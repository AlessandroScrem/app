use std::collections::{HashMap, HashSet};

use legion::{Entity, EntityStore, World};

use crate::{
    EntityRawU64, Globals,
    assets::MeshId,
    ecs::components::{
        BoundingBoxComponent, GlobalModelComponent, HierarchyComponent, Hidden, LightComponent,
        MeshComponent,
    },
    math::{Mat4, Point3f, Vec3, Vec4},
    renderer::uniform::LightUniform,
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
    pub position: Point3f,
}

impl LightRenderObject {
    const SIZE: f32 = 20.0;
    const NEAR: f32 = 0.1;
    const FAR: f32 = 100.0;

    fn get_proj_matrix() -> Mat4 {
        crate::math::ortho(
            -Self::SIZE,
            Self::SIZE,
            -Self::SIZE,
            Self::SIZE,
            Self::NEAR,
            Self::FAR,
        )
    }

    pub fn get_view_proj_matrix(&self) -> Mat4 {
        Self::get_proj_matrix() * Self::view_matrix(self.position)
    }

    pub fn view_matrix<P>(position: P) -> Mat4
    where
        P: Into<Point3f>,
    {
        Mat4::look_at_rh(position.into(), Point3f::new(0.0, 0.0, 0.0), Vec3::unit_y())
    }
}

impl From<&LightRenderObject> for LightUniform {
    fn from(value: &LightRenderObject) -> Self {
        Self {
            color: value.light.color,
            directional: value.light.directional.into(),
            position: value.position.into(),
            cast_shadow: value.light.cast_shadow.into(),
            entity_id: value.entity_id,
            view_proj: value.get_view_proj_matrix().into(),
            ..Default::default()
        }
    }
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
        // Visibility is shared by all extracted object types. Memoizing it avoids
        // walking the same parent chains separately for meshes, lights and bounds.
        let mut visibility = HashMap::new();
        let meshes = extract_meshes(world, &mut visibility);
        let lights = extract_lights(world, &mut visibility);
        let bboxes = extract_bbox(world, globals.bbox_enable, &mut visibility);

        Self {
            meshes,
            bboxes,
            lights,
        }
    }
}

fn is_hidden_cached(
    world: &World,
    entity: Entity,
    cache: &mut HashMap<Entity, bool>,
) -> bool {
    if let Some(hidden) = cache.get(&entity) {
        return *hidden;
    }

    let mut path = Vec::new();
    let mut visited = HashSet::new();
    let mut current = Some(entity);
    let mut hidden = false;

    while let Some(current_entity) = current {
        if let Some(cached) = cache.get(&current_entity) {
            hidden = *cached;
            break;
        }
        if !visited.insert(current_entity) {
            // Malformed cyclic hierarchy: do not recurse indefinitely.
            hidden = false;
            break;
        }

        path.push(current_entity);
        let Ok(entry) = world.entry_ref(current_entity) else {
            hidden = false;
            break;
        };

        if entry.get_component::<Hidden>().is_ok() {
            hidden = true;
            break;
        }

        current = entry
            .get_component::<HierarchyComponent>()
            .ok()
            .and_then(|hierarchy| hierarchy.parent);
    }

    for entity in path {
        cache.insert(entity, hidden);
    }
    hidden
}

fn extract_meshes(
    world: &World,
    visibility: &mut HashMap<Entity, bool>,
) -> Vec<MeshRenderObject> {
    use legion::IntoQuery;
    let mut query = <(Entity, &MeshComponent, &GlobalModelComponent)>::query();

    let mut meshes = Vec::new();
    for (entity, mesh, transform) in query.iter(world) {
        if is_hidden_cached(world, *entity, visibility) {
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

fn extract_lights(
    world: &World,
    visibility: &mut HashMap<Entity, bool>,
) -> Vec<LightRenderObject> {
    use legion::IntoQuery;
    let mut query = <(Entity, &LightComponent, &GlobalModelComponent)>::query();

    let mut lights = Vec::new();
    for (entity, light, transform) in query.iter(world) {
        if is_hidden_cached(world, *entity, visibility) {
            continue;
        }

        let pos: [f32; 3] = (transform.mat * Vec4::new(0.0, 0.0, 0.0, 1.0))
            .truncate()
            .into();

        lights.push(LightRenderObject {
            entity_id: entity.as_raw_u64(),
            light: light.clone(),
            position: pos.into(),
        });
    }
    lights
}

fn extract_bbox(
    world: &World,
    bbox_enable: bool,
    visibility: &mut HashMap<Entity, bool>,
) -> Vec<BboxRenderObject> {
    if !bbox_enable {
        return Vec::new();
    }

    use legion::IntoQuery;
    let mut query = <(Entity, &BoundingBoxComponent, &GlobalModelComponent)>::query();

    let mut bboxes = Vec::new();
    query.for_each(world, |(entity, bbox, transform)| {
        if is_hidden_cached(world, *entity, visibility) {
            return;
        }

        bboxes.push(BboxRenderObject {
            entity_id: entity.as_raw_u64(),
            bbox: bbox.clone(),
            transform: transform.mat,
        });
    });
    bboxes
}

#[cfg(test)]
mod tests {
    use super::is_hidden_cached;
    use crate::ecs::components::{Hidden, HierarchyComponent};
    use legion::World;
    use std::collections::HashMap;

    #[test]
    fn visibility_cache_inherits_hidden_state_from_parent() {
        let mut world = World::default();
        let parent = world.push((HierarchyComponent::default(), Hidden));
        let child = world.push(HierarchyComponent {
            parent: Some(parent),
            children: Vec::new(),
        });
        let mut cache = HashMap::new();

        assert!(is_hidden_cached(&world, child, &mut cache));
        assert!(cache.get(&parent).copied().unwrap_or(false));
        assert!(cache.get(&child).copied().unwrap_or(false));
    }

    #[test]
    fn visibility_cache_reuses_ancestor_results() {
        let mut world = World::default();
        let parent = world.push(HierarchyComponent::default());
        let child = world.push(HierarchyComponent {
            parent: Some(parent),
            children: Vec::new(),
        });
        let mut cache = HashMap::new();

        assert!(!is_hidden_cached(&world, child, &mut cache));
        assert_eq!(cache.get(&parent), Some(&false));
        assert_eq!(cache.get(&child), Some(&false));
    }
}
