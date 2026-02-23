mod app;
pub(crate) mod assets;
pub(crate) mod bounding_box;
mod camera;
mod engine;
pub(crate) mod entities;
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

// /// Facade pubblica: crea e fa partire l'applicazione internamente
// pub fn run_app() -> Result<(), Box<dyn std::error::Error>> {
//     // qui usiamo un tipo concreto interno che implementa tutti i trait necessari
//     // supponiamo che MyAppImpl sia già definito nel crate
//     let app = engine::MyApplication::<app::App>::new_with_size(800, 600);
//     app.run()
// }

pub(crate) mod prelude {
    pub(crate) use crate::app::App;
    pub(crate) use crate::app::domain::*;
    pub(crate) use crate::assets::asset_manager::AssetManager;
    pub(crate) use crate::assets::material_asset;
    pub(crate) use crate::bounding_box::BoundingBox;
    pub(crate) use crate::camera::Camera;
    pub(crate) use crate::entities::components::*;
    pub(crate) use crate::renderer::Renderer;
    pub(crate) use crate::renderer::uniform;
    pub(crate) use crate::timestep;
    pub(crate) use crate::ui::*;
    pub(crate) use log::{debug, error, info, trace, warn};
}

pub(crate) use prelude::*;

pub(crate) mod math {
    pub(crate) fn vec3_min(a: &Vec3, b: &Vec3) -> Vec3 {
        Vec3 {
            x: a.x.min(b.x),
            y: a.y.min(b.y),
            z: a.z.min(b.z),
        }
    }

    pub(crate) fn vec3_max(a: &Vec3, b: &Vec3) -> Vec3 {
        Vec3 {
            x: a.x.max(b.x),
            y: a.y.max(b.y),
            z: a.z.max(b.z),
        }
    }
    use cgmath::*;
    pub(crate) type Mat4 = Matrix4<f32>;
    pub(crate) type Vec2 = Vector2<f32>;
    pub(crate) type Vec3 = Vector3<f32>;
    pub(crate) type Vec4 = Vector4<f32>;
    pub(crate) type Point3f = Point3<f32>;
    pub(crate) type Quat = Quaternion<f32>;
    pub(crate) use cgmath::{
        Angle, Array, Deg, EuclideanSpace, Euler, InnerSpace as _, Matrix as _, One, Rad,
        Rotation3 as _, SquareMatrix as _, Zero,
        num_traits::{one, zero},
        perspective, vec3, vec4,
    };
}

use crate::assets::MaterialDesc;

pub(crate) mod colors {
    pub(crate) const SILVER: [f32; 3] = [0.7, 0.7, 0.7];
    pub(crate) const CYAN_COLOR: [f32; 3] = [0.0, 1.0, 1.0];
    pub(crate) const YELLOW_COLOR: [f32; 3] = [1.0, 0.5, 1.0];
    pub(crate) const LIGHT_YELLOW_COLOR: [f32; 3] = [1.0, 0.9, 0.5];
    pub(crate) const RED_COLOR: [f32; 3] = [0.8, 0.3, 0.2];
    pub(crate) const GREEN_COLOR: [f32; 3] = [0.2, 0.8, 0.3];
    pub(crate) const BLUE_COLOR: [f32; 3] = [0.2, 0.3, 0.8];
    pub(crate) const CLEAR_COLOR: [f32; 3] = [0.1, 0.1, 0.1];
}

#[derive(Clone, Debug)]
pub(crate) struct Globals {
    pub(crate) ibl_enable: bool,
    pub(crate) skybox_enable: bool,
    pub(crate) exposure: f32,
    pub(crate) ibl_intensity: f32,
    pub(crate) tonemap_filter: u32,
    pub(crate) axis_enable: bool,
    pub(crate) bbox_enable: bool,
    pub(crate) bbox_axis_aligned: bool,
    pub(crate) debug_code: u32,
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
