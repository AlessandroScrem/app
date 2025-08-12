mod camera;
mod renderer;
mod app;
mod application_handler;
mod scene;
mod entities;
pub mod input;
pub mod systems;
pub mod resources;
pub mod assets;
pub mod transform;

pub mod prelude {
    pub use super::app::App;
    pub use crate::camera::Camera;
    pub use crate::renderer::Renderer;
    pub use crate::renderer::uniform::CameraUniform;
    pub use crate::renderer::imgui_tools;
}

#[derive(Clone, Copy, Debug)]
pub struct DeltaTime(pub f32);

