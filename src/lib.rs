mod app;
mod assets;
mod bounding_box;
mod camera;
mod engine;
mod error;
mod globals;
mod gpu;
mod input;
mod picking;
mod renderer;
mod scene;
mod systems;
mod test_utils;
mod timer;
mod timestep;
mod transform;
mod ui;

pub(crate) mod entities;

pub use entities::EntityRawU64;

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
    pub use crate::bounding_box::BoundingBox;
    pub use crate::camera::Camera;
    pub(crate) use log::{debug, error, info, trace, warn};
}

pub(crate) use prelude::*;

pub(crate) use globals::Globals;

#[macro_export]
macro_rules! impl_debug_drop {
    ($t:ty) => {
        impl Drop for $t {
            fn drop(&mut self) {
                println!("Dropped {}", std::any::type_name::<Self>());
            }
        }
    };
}

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
