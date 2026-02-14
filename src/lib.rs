mod app;
mod engine;
pub mod assets;
pub mod bounding_box;
mod camera;
pub mod entities;
pub mod input;
mod picking;
pub mod renderer;
mod scene;
mod systems;
pub mod test_utils;
mod timer;
pub mod timestep;
mod transform;
pub mod ui;


pub mod prelude {
    pub use crate::assets::material_asset;
    pub use crate::assets::asset_manager::AssetManager;
    pub use crate::bounding_box::BoundingBox;
    pub use crate::camera::Camera;
    pub use crate::entities::components::*;
    pub use crate::renderer::Renderer;
    pub use crate::renderer::uniform;
    pub use crate::timestep;
    pub use crate::ui::*;
    pub use crate::app::domain::*;
    pub use app::App;
    pub use engine::MyApplication;
    pub use log::{debug, error, info, trace, warn};
}

pub use prelude::*;

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
    pub use cgmath::{
        Angle, Array, Deg, EuclideanSpace, Euler, InnerSpace as _, Matrix as _, One, Rad,
        Rotation3 as _, SquareMatrix as _, Zero,
        num_traits::{one, zero},
        perspective, vec3, vec4,
    };
}


use crate::assets::MaterialDesc;

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




#[derive(Clone, Debug)]
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
