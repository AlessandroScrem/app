mod camera;
mod renderer;
mod app;
mod application_handler;
mod model_reader;
mod scene;
mod entities;
mod input;

pub mod prelude {
    pub use super::app::App;
    pub use crate::camera::Camera;
    pub use crate::renderer::renderer_impl::Renderer;
}

#[derive(Clone, Copy, Debug)]
pub struct DeltaTime(pub f32);

