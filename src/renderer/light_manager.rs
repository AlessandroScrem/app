use crate::{assets::texture::Texture, resources::gpu_manager::GPUResourceManager};
use std::sync::Arc;
use wgpu::util::DeviceExt;

pub struct LightManager {
    pub light_texture_bind_group: wgpu::BindGroup,
    pub light_uniform_buffer: wgpu::Buffer,
    pub light_uniform_bind_group: wgpu::BindGroup,
}

impl LightManager {
    pub fn new(
        gpu_resource_manager: &GPUResourceManager,
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
    ) -> Self {
        let diffuse_bytes = include_bytes!("../../assets/core/lightbulb-icon32.png");
        let light_texture = Texture::new(&device, &queue, diffuse_bytes.to_vec(), false);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let light_texture_bind_group_layout = gpu_resource_manager.get_layout("light_texture");

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

        let light_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Uniform Buffer"),
            contents: bytemuck::cast_slice(&[crate::Light::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group_layout = gpu_resource_manager.get_layout("light");
        let light_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_uniform_buffer.as_entire_binding(),
            }],
            label: Some("Light uniform Bind Group"),
        });

        Self {
            light_texture_bind_group,
            light_uniform_buffer,
            light_uniform_bind_group,
        }
    }
}
