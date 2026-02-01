use super::{
    texture::Texture,
    gpu_manager::{GpuManager, LayoutKind},
};

pub struct LightManager {
    pub light_texture_bind_group: wgpu::BindGroup,
}

impl LightManager {
    pub fn new(
        gpu_manager: &GpuManager,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let light_texture = {
            let buffer = include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/core/lightbulb-icon32.png"
            ));
            Texture::new(&device, &queue, buffer, wgpu::TextureFormat::Rgba8UnormSrgb)
        };

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let light_texture_bind_group_layout =
            gpu_manager.get_layout(LayoutKind::LightTexture);

        let light_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&light_texture.view),
                },
            ],
            label: Some("light texture_bind_group"),
        });

        Self {
            light_texture_bind_group,
        }
    }
}
