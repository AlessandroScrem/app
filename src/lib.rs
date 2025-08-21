use std::{path::Path, sync::Arc};

use crate::{assets::texture_manager::TextureManager, resources::gpu_manager::GPUResourceManager};

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

pub struct SkyboxBindGroup(pub wgpu::BindGroup);

pub fn create_skybox(resources: &mut legion::Resources) {
    #[rustfmt::skip] let f0 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/skybox/right.png"));
    #[rustfmt::skip] let f1 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/skybox/left.png"));
    #[rustfmt::skip] let f2 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/skybox/top.png"));
    #[rustfmt::skip] let f3 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/skybox/bottom.png"));
    #[rustfmt::skip] let f4 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/skybox/front.png"));
    #[rustfmt::skip] let f5 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/skybox/back.png"));

    let skybox_bind_group = {
        let mut texture_manager = resources.get_mut::<TextureManager>().unwrap();
        let device = resources.get::<wgpu::Device>().unwrap();

        let cube = texture_manager.create_cubemap(
            f0,
            f1,
            f2,
            f3,
            f4,
            f5,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        );

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let gpu_resource_manager = resources.get::<Arc<GPUResourceManager>>().unwrap();
        let skybox_bind_group_layout = gpu_resource_manager.get_layout("skybox");

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &skybox_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&cube.view),
                },
            ],
            label: Some("skybox_bind_group"),
        })
    };

    resources.insert(SkyboxBindGroup(skybox_bind_group));
}

pub fn create_hdr(resources: &mut legion::Resources) {
    #[rustfmt::skip] let f0 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/core/clarens_night_02_2k.hdr"));

    let mut texture_manager = resources.get_mut::<TextureManager>().unwrap();

    texture_manager.get_or_create(f0, wgpu::TextureFormat::Rgba16Float);
}
