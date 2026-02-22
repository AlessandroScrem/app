use crate::renderer::GpuTexture;

use super::gpu_manager::{GpuManager, LayoutKind};

pub struct LightManager {
    pub light_texture_bind_group: wgpu::BindGroup,
}

pub const LIGHT_BULB_PATH: &'static str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/core/lightbulb-icon32.png"
);

impl LightManager {
    pub fn new(
        light_texture: &GpuTexture,
        gpu_manager: &GpuManager,
        device: &wgpu::Device,
    ) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let light_texture_bind_group_layout = gpu_manager.get_layout(LayoutKind::LightTexture);

        let light_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&light_texture.texture.view),
                },
            ],
            label: Some("light texture_bind_group"),
        });

        Self {
            light_texture_bind_group,
        }
    }
}
