mod app;
mod assets;
mod bounding_box;
mod camera;
mod ecs;
mod editor;
mod engine;
mod error;
mod globals;
mod gpu;
mod input;
mod math;
mod renderer;
mod scene;
mod test_utils;
mod timer;
mod timestep;
mod ui;

pub use ecs::entity_id::EntityRawU64;

pub(crate) use bounding_box::BoundingBox;
pub(crate) use camera::Camera;
pub(crate) use globals::Globals;

pub(crate) mod prelude {
    pub use log::{debug, error, info, trace, warn};
}

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

#[macro_export]
macro_rules! impl_debug_drop {
    ($t:ty) => {
        impl Drop for $t {
            fn drop(&mut self) {
                log::info!("Dropped {}", std::any::type_name::<Self>());
            }
        }
    };
}

#[macro_export]
macro_rules! asset_bytes {
    ($path:expr) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $path))
    };
}

#[macro_export]
macro_rules! asset_path {
    ($path:expr) => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $path)
    };
}

#[macro_export]
macro_rules! project_path {
    ($path:expr) => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/", $path)
    };
}

pub(crate) mod colors {
    pub const CYAN_COLOR: [f32; 3] = [0.0, 1.0, 1.0];
    pub const GREEN_COLOR: [f32; 3] = [0.2, 0.8, 0.3];
    pub const RED_COLOR: [f32; 3] = [0.8, 0.3, 0.2];
    pub const BLUE_COLOR: [f32; 3] = [0.2, 0.3, 0.8];
    // pub const BACKGROUND_COLOR: [f32; 3] = [0.188, 0.208, 0.259]; // from GltfViewer
    // pub const SILVER: [f32; 3] = [0.7, 0.7, 0.7];
    // pub const YELLOW_COLOR: [f32; 3] = [1.0, 0.5, 1.0];
    // pub const LIGHT_YELLOW_COLOR: [f32; 3] = [1.0, 0.9, 0.5];
    // pub const CLEAR_COLOR: [f32; 3] = [0.1, 0.1, 0.1];
}
