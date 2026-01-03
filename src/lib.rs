mod app;
mod application_handler;
pub mod assets;
mod camera;
pub mod entities;
pub mod input;
mod picking;
pub mod renderer;
mod scene;
mod systems;
pub mod test_utils;
pub mod timestep;
mod transform;

pub mod prelude {
    pub use super::app::App;
    pub use crate::camera::{Camera, center_camera_to_bounding_box};
    pub use crate::renderer::Renderer;
    pub use crate::renderer::ui;
    pub use crate::timestep;
    pub use log::{debug, error, info, trace, warn};
}

pub mod math {
    pub fn vec3_min(a: &Vec3, b: &Vec3) -> Vec3 {
        Vec3 {
            x: a.x.min(b.x),
            y: a.y.min(b.y),
            z: a.z.min(b.z),
        }
    }

    pub fn vec3_max(a: &Vec3, b: &Vec3) -> Vec3 {
        Vec3 {
            x: a.x.max(b.x),
            y: a.y.max(b.y),
            z: a.z.max(b.z),
        }
    }
    use cgmath::*;
    pub type Mat4 = Matrix4<f32>;
    pub type Vec2 = Vector2<f32>;
    pub type Vec3 = Vector3<f32>;
    pub type Vec4 = Vector4<f32>;
    pub type Point3f = Point3<f32>;
    pub type Quat = Quaternion<f32>;
    pub use cgmath::{Angle, Deg, Euler, Rad, Zero, perspective, vec3, vec4};
    pub use cgmath::{
        EuclideanSpace, InnerSpace as _, Matrix as _, Rotation3 as _, SquareMatrix as _,
    };
}

use std::{collections::VecDeque, path::PathBuf};

use crate::{assets::material_manager::{MaterialId, MaterialPBR}, entities::bounding_box::BoundingBox};
use legion::Entity;
use math::*;

pub mod colors {
    pub const SILVER: [f32; 3] = [0.7, 0.7, 0.7];
    pub const CYAN_COLOR: [f32; 3] = [0.0, 1.0, 1.0];
    pub const YELLOW_COLOR: [f32; 3] = [1.0, 0.5, 1.0];
    pub const LIGHT_YELLOW_COLOR: [f32; 3] = [1.0, 0.9, 0.5];
    pub const RED_COLOR: [f32; 3] = [0.8, 0.3, 0.2];
    pub const GREEN_COLOR: [f32; 3] = [0.2, 0.8, 0.3];
    pub const BLUE_COLOR: [f32; 3] = [0.2, 0.3, 0.8];
    pub const CLEAR_COLOR: [f32; 3] = [0.1, 0.1, 0.1];
}
#[derive(Default)]
pub struct UiComponentView{
    tag: Option<TagComponent>,
    mesh: Option<MeshComponent>,
    transform: Option<TransformComponent>,
    bounding_box: Option<BoundingBoxComponent>,
    material: Option<MaterialPBR>,
    light: Option<LightComponent>,
    dirty: bool
}

pub enum DomainEvent {
    RemoveEntity(Entity),
    LoadGltf(PathBuf),
    AddParent(Entity),
}

pub struct DomainEvents {
    pub queue: VecDeque<DomainEvent>,
}

#[derive(Clone, Copy, Debug)]
pub struct Globals {
    pub ibl_enable: bool,
    pub skybox_enable: bool,
    pub exposure: f32,
    pub ibl_intensity: f32,
    pub tonemap_filter: u32,
    pub axis_enable: bool,
    pub bbox_enable: bool,
    pub bbox_axis_aligned: bool,
    pub debug_code: u32,
}
impl Default for Globals {
    fn default() -> Self {
        Self {
            ibl_enable: true,
            skybox_enable: true,
            exposure: 1.0,
            ibl_intensity: 1.0,
            tonemap_filter: 0,
            axis_enable: true,
            bbox_enable: false,
            bbox_axis_aligned: false,
            debug_code: 0,
        }
    }
}

// Ecs Components
#[derive(Default, Clone)]
pub struct LightComponent {
    pub data: renderer::LightUniform,
}

#[derive(Default, Clone)]
pub struct MeshComponent {
    pub handle: usize,
    pub mat_handle: MaterialId,
}

#[derive(Clone)]
pub struct TransformComponent {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
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
