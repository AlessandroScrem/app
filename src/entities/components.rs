use crate::math::*;
use crate::prelude::*;
use legion::Entity;

use crate::assets::MeshId;

// Ecs Components
#[derive(Default, Clone)]
pub(crate) struct LightComponent {
    pub(crate) data: uniform::LightUniform,
}

#[derive(Default, Clone)]
pub(crate) struct MeshComponent {
    pub(crate) handle: MeshId,
}

#[derive(Clone)]
pub(crate) struct TransformComponent {
    pub(crate) position: [f32; 3],
    pub(crate) rotation: [f32; 3],
    pub(crate) scale: [f32; 3],
}

impl TransformComponent {
    pub(crate) fn from_gltf(g_node: &gltf::Node<'_>) -> Self {
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
pub(crate) struct GlobalModelComponent {
    pub(crate) mat: Mat4,
}

impl Default for GlobalModelComponent {
    fn default() -> Self {
        Self {
            mat: Mat4::identity(),
        }
    }
}

#[derive(Default, Clone)]
pub(crate) struct TagComponent {
    pub(crate) name: String,
}

#[derive(Clone)]
pub(crate) struct BoundingBoxComponent {
    pub(crate) global_bounding_box: BoundingBox,
    pub(crate) bounding_box: BoundingBox,
}

#[derive(Default, Clone)]
pub(crate) struct HierarchyComponent {
    pub(crate) parent: Option<Entity>,
    pub(crate) children: Vec<Entity>,
}
