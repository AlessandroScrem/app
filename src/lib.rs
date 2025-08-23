use std::{
    path::Path,
    sync::{Arc, OnceLock},
};

use crate::assets::texture_manager::TextureManager;

mod app;
mod application_handler;
pub mod assets;
mod camera;
mod entities;
pub mod input;
mod renderer;
pub mod resources;
mod scene;
pub mod systems;
pub mod transform;

pub mod prelude {
    pub use super::app::App;
    pub use crate::camera::Camera;
    pub use crate::renderer::Renderer;
    pub use crate::renderer::imgui_tools;
    pub use crate::renderer::uniform::CameraUniform;
}

static DEVICE_AND_QUEUE: OnceLock<(Arc<wgpu::Device>, Arc<wgpu::Queue>)> = OnceLock::new();
pub fn get_device_and_queue() -> &'static (Arc<wgpu::Device>, Arc<wgpu::Queue>) {
    DEVICE_AND_QUEUE.get_or_init(|| {
        let instance = wgpu::Instance::default();
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .unwrap();

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).unwrap();

        (Arc::new(device), Arc::new(queue))
    })
}

#[derive(Clone, Copy, Debug)]
pub struct DeltaTime(pub f32);

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Light {
    color: [f32; 3],
    directional: u32,
    position: [f32; 3],
    cast_shadow: u32,
}
impl Default for Light {
    fn default() -> Self {
        Self {
            color: [1.0, 1.0, 1.0],
            cast_shadow: 0,
            directional: 1,
            position: [0.0, 0.0, -1.0],
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

pub struct TransformComponent {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
}

pub struct TagComponent {
    pub name: String,
}


pub fn create_hdr(resources: &mut legion::Resources) {
    #[rustfmt::skip] let f0 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/core/clarens_night_02_2k.hdr"));

    let mut texture_manager = resources.get_mut::<TextureManager>().unwrap();

    texture_manager.get_or_create(f0, wgpu::TextureFormat::Rgba16Float);
}
