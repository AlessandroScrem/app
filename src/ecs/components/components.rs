use crate::math::*;
use crate::prelude::*;
use legion::Entity;

use crate::assets::MeshId;
use crate::renderer::uniform::*;

// Ecs Components
#[derive(Clone)]
pub struct LightComponent {
    pub color: [f32; 3],
    pub directional: bool,
    position: [f32; 3],
    pub cast_shadow: bool,
    pub entity_id: u64,
    pub enabled: bool,
    view_matrix: Mat4,
    proj_matrix: Mat4,
}
impl Default for LightComponent {
    fn default() -> Self {
        const WHITE: [f32; 3] = [1.0, 1.0, 1.0];
        const POSITION: [f32; 3] = [0.0, 0.0, -1.0];
        const SIZE: f32 = 20.0;
        const NEAR: f32 = 2.0;
        const FAR: f32 = 200.0;
        let proj_matrix = ortho(-SIZE, SIZE, -SIZE, SIZE, NEAR, FAR);
        let view_matrix = Self::view_matrix(POSITION);

        Self {
            color: WHITE,
            enabled: true,
            cast_shadow: false,
            directional: true,
            position: POSITION,
            entity_id: 0,
            proj_matrix,
            view_matrix,
        }
    }
}

impl LightComponent {
    pub fn get_view_proj_matrix(&self) -> Mat4 {
        self.proj_matrix * self.view_matrix
    }

    pub fn get_position(&self) ->[f32;3] {
        self.position
    }

    pub fn update_position<P>(&mut self, position: P)
        where
        P: Into<[f32;3]>,
    {
        self.position = position.into();
        self.update_view_matrix();
    }

    fn update_view_matrix(&mut self) {
        self.view_matrix = Self::view_matrix(self.position);
    }

    fn view_matrix<P>(position: P) -> Mat4
    where
        P: Into<Point3f>,
    {
        let eye: Point3f = position.into();

        Mat4::look_at_rh(eye, Point3f::new(0.0, 0.0, 0.0), Vec3::unit_y())
    }
}

impl From<&LightComponent> for LightUniform {
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

impl Default for TransformComponent {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}

impl TransformComponent {
    pub fn compute_model_matrix(&self) -> Mat4 {
        // converte la rotazione xyz "Radianti" in Quaternion
        fn to_quat(r: &[f32; 3]) -> Quat {
            let euler = Euler::new(Rad(r[0]), Rad(r[1]), Rad(r[2]));
            Quat::from(euler)
        }

        let translation = Vec3::from(self.position);
        let rotation = to_quat(&self.rotation);
        let scale = Vec3::from(self.scale);

        let t = Mat4::from_translation(translation);
        let r = Mat4::from(rotation);
        let s = Mat4::from_nonuniform_scale(scale.x, scale.y, scale.z);

        t * r * s
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
