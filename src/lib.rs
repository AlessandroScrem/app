mod app;
mod application_handler;
mod scene;
mod camera;
mod picking;
mod systems;
mod transform;
pub mod renderer;
pub mod assets;
pub mod entities;
pub mod input;
pub mod timestep;
pub mod test_utils;

pub mod prelude {
    pub use super::app::App;
    pub use crate::camera::Camera;
    pub use crate::renderer::Renderer;
    pub use crate::renderer::imgui_tools;
    pub use crate::timestep;
    pub use log::{debug, error, info, trace, warn};
}

pub mod math {
    use cgmath::*;
    pub type Mat4 = Matrix4<f32>;
    pub type Vec2 = Vector2<f32>;
    pub type Vec3 = Vector3<f32>;
    pub type Vec4 = Vector4<f32>;
    pub type Point3f = Point3<f32>;
    pub type Quat = Quaternion<f32>;
    pub use cgmath::{Deg, Euler, Rad, perspective, vec3, vec4, Zero};
    pub use cgmath::{EuclideanSpace, InnerSpace as _, Matrix as _, SquareMatrix as _, Rotation3 as _};
}

use math::*;
use legion::Entity;
use crate::{assets::vertexdata::LinesVertexData, entities::bounding_box::BoundingBox};

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

#[derive(Clone, Copy, Debug)]
pub struct Globals {
    pub ibl_enable: bool,
    pub skybox_enable: bool,
    pub exposure: f32,
    pub tonemap_filter: u32,
    pub axis_enable: bool,
    pub bbox_enable: bool,
}
impl Default for Globals {
    fn default() -> Self {
        Self {
            ibl_enable: true,
            skybox_enable: true,
            exposure: 1.0,
            tonemap_filter: 0,
            axis_enable: true,
            bbox_enable: true,
        }
    }
}

///shader: [pbr, blinnphong, light]
#[repr(C, align(16))]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Light {
    color: [f32; 3],
    directional: u32,
    position: [f32; 3],
    cast_shadow: u32,
    entity_id: u64,
    pad2: [i32; 2],
}
impl Default for Light {
    fn default() -> Self {
        Self {
            color: [1.0, 1.0, 1.0],
            cast_shadow: 0,
            directional: 1,
            position: [0.0, 0.0, -1.0],
            entity_id: 0,
            pad2: [0, 0],
        }
    }
}

// Ecs Components
#[derive(Default, Clone)]
pub struct LightComponent {
    pub data: Light,
}

pub struct MeshComponent {
    data: assets::mesh::Mesh,
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

impl From<Mat4> for GlobalModelComponent {
    fn from(value: Mat4) -> Self {
        Self { mat: value }
    }
}

pub struct TagComponent {
    pub name: String,
}

pub struct BoundingBoxComponent {
    pub vertices: [LinesVertexData; 24],
    pub bounding_box: BoundingBox,
    pub vertex_buffer: wgpu::Buffer,
}

#[derive(Clone)]
pub struct HierarchyComponent {
    pub parent: Option<Entity>,
    pub children: Vec<Entity>,
}
