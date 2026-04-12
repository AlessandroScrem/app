use crate::math::*;
use crate::prelude::*;
use legion::Entity;

use crate::assets::MeshId;

// Ecs Components
#[derive(Clone)]
pub struct LightComponent {
    pub color: [f32; 3],
    pub directional: bool,
    pub position: [f32; 3],
    pub cast_shadow: bool,
    pub entity_id: u64,
    pub enabled: bool,
}
impl Default for LightComponent {
    fn default() -> Self {
        Self {
            color: [1.0, 1.0, 1.0],
            enabled: true,
            cast_shadow: false,
            directional: true,
            position: [0.0, 0.0, -1.0],
            entity_id: 0,
        }
    }
}

impl From<&LightComponent> for uniform::LightUniform {
    fn from(value: &LightComponent) -> Self {
        Self {
            color: value.color,
            directional: value.directional.into(),
            position: value.position,
            cast_shadow: value.cast_shadow.into(),
            entity_id: value.entity_id,
            enabled: value.enabled.into(),
            ..Default::default()
        }
    }
}

#[derive(Default, Clone)]
pub struct MeshComponent {
    pub handle: MeshId,
}

#[derive(Clone)]
pub struct TransformComponent {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
}

impl TransformComponent {
    pub fn from_gltf(g_node: &gltf::Node<'_>) -> Self {
        let (position, r, scale) = g_node.transform().decomposed();
        let quat = Quat::new(r[3], r[0], r[1], r[2]);
        let euler = Euler::from(quat);
        let rotation = [euler.x.0, euler.y.0, euler.z.0];
        Self {
            position,
            rotation,
            scale,
        }
    }
}

#[derive(Clone)]
pub struct GlobalModelComponent {
    pub mat: Mat4,
}

impl Default for GlobalModelComponent {
    fn default() -> Self {
        Self {
            mat: Mat4::identity(),
        }
    }
}

#[derive(Default, Clone)]
pub struct TagComponent {
    pub name: String,
}

#[derive(Clone)]
pub struct BoundingBoxComponent {
    pub global_bounding_box: BoundingBox,
    pub bounding_box: BoundingBox,
}

#[derive(Default, Clone)]
pub struct HierarchyComponent {
    pub parent: Option<Entity>,
    pub children: Vec<Entity>,
}
