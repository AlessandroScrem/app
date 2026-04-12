pub mod entities;

mod app;
pub(crate) mod assets;
pub(crate) mod bounding_box;
mod camera;
mod engine;
pub(crate) mod input;
mod picking;
pub(crate) mod renderer;
mod scene;
mod systems;
pub(crate) mod test_utils;
mod timer;
pub(crate) mod timestep;
mod transform;
pub(crate) mod ui;
pub(crate) mod error;
pub(crate) mod  gpu;

pub struct Engine {
    inner: engine::MyApplication<app::App>,
}

impl Engine {
    pub fn new_with_size(width: u32, height: u32) -> Self {
        Self {
            inner: engine::winit_bridge::MyApplication::<app::App>::new_with_size(width, height),
        }
    }
    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        self.inner.run()
    }
}

pub(crate) mod prelude {
    pub use crate::assets::asset_manager::AssetManager;
    pub(crate) use crate::assets::material_asset;
    pub use crate::bounding_box::BoundingBox;
    pub use crate::camera::Camera;
    pub use crate::entities::components::*;
    pub use crate::renderer::SceneRenderer;
    pub(crate) use crate::renderer::uniform;
    pub(crate) use crate::ui::*;
    pub(crate) use log::{debug, error, info, trace, warn};
    pub(crate) use crate::assets::MaterialDesc;
    pub use timer::Timer;
    pub use error::*;
}

pub(crate) use prelude::*;

#[allow(unused_imports)]
pub(crate) mod math {
    use cgmath::*;
    pub type Mat3 = Matrix3<f32>;
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

    // pub fn vec3_min(a: &Vec3, b: &Vec3) -> Vec3 {
    //     Vec3 {
    //         x: a.x.min(b.x),
    //         y: a.y.min(b.y),
    //         z: a.z.min(b.z),
    //     }
    // }

    // pub fn vec3_max(a: &Vec3, b: &Vec3) -> Vec3 {
    //     Vec3 {
    //         x: a.x.max(b.x),
    //         y: a.y.max(b.y),
    //         z: a.z.max(b.z),
    //     }
    // }
}


pub(crate) mod colors {
    pub const CYAN_COLOR: [f32; 3] = [0.0, 1.0, 1.0];
    // pub const BACKGROUND_COLOR: [f32; 3] = [0.188, 0.208, 0.259]; // from GltfViewer
    // pub const SILVER: [f32; 3] = [0.7, 0.7, 0.7];
    // pub const YELLOW_COLOR: [f32; 3] = [1.0, 0.5, 1.0];
    // pub const LIGHT_YELLOW_COLOR: [f32; 3] = [1.0, 0.9, 0.5];
    // pub const RED_COLOR: [f32; 3] = [0.8, 0.3, 0.2];
    // pub const GREEN_COLOR: [f32; 3] = [0.2, 0.8, 0.3];
    // pub const BLUE_COLOR: [f32; 3] = [0.2, 0.3, 0.8];
    // pub const CLEAR_COLOR: [f32; 3] = [0.1, 0.1, 0.1];
}

#[derive(Clone, Debug)]
pub(crate) struct Globals {
    pub mips_cs: bool,
    pub light_enable: bool,
    pub ibl_enable: bool,
    pub skybox_enable: bool,
    pub skybox_enable_blur: bool,
    pub env_rotation: f32,
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
            mips_cs: false, 
            light_enable: false,
            ibl_enable: true,
            skybox_enable: true,
            skybox_enable_blur: true,
            exposure: 1.0,
            env_rotation: 0.0,
            ibl_intensity: 1.0,
            tonemap_filter: 0,
            axis_enable: true,
            bbox_enable: false,
            bbox_axis_aligned: false,
            debug_code: 0,
        }
    }
}
