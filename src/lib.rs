mod camera;
mod renderer;
mod app;
mod application_handler;
mod model_reader;
mod scene;
mod entities;
pub mod input;

pub mod prelude {
    pub use super::app::App;
    pub use crate::camera::Camera;
    pub use crate::renderer::renderer::Renderer;
    pub use crate::renderer::uniform::CameraUniform;
    pub use crate::renderer::egui_tools;
}

#[derive(Clone, Copy, Debug)]
pub struct DeltaTime(pub f32);

