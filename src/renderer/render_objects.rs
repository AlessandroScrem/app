use legion::{Entity, EntityStore, World};

use crate::{
    EntityRawU64, Globals,
    assets::MeshId,
    ecs::components::{
        BoundingBoxComponent, GlobalModelComponent, Hidden, HierarchyComponent, LightComponent,
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
    let mut query = <(Entity, &LightComponent, &GlobalModelComponent)>::query();

    let mut lights = Vec::new();
    for (entity, light, transform) in query.iter(world) {
        if is_hidden(world, *entity) {
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
