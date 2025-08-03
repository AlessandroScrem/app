use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::{assets::texture::Texture, resources::gpu_manager::GPUResourceManager};

pub struct Material {
    pub main_texture: String,
    pub roughness_texture: String,
    pub normal_texture: String,
    pub roughness: f32,
    pub metallic: f32,
    pub roughness_override: f32,
    pub metallic_override: f32,
    pub color: cgmath::Vector4<f32>,
    pub textures: std::collections::HashMap<String, Texture>,
}

pub struct MaterialManager {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    gpu_manager: Arc<GPUResourceManager>,
    materials: HashMap<PathBuf, Material>,
}

impl MaterialManager {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        gpu_manager: Arc<GPUResourceManager>,
    ) -> Self {
        Self {
            device,
            queue,
            gpu_manager,
            materials: HashMap::new(),
        }
    }

    pub fn add_material(&mut self, mut material: Material, path: PathBuf) {
        let texture = get_texture(path.join(&material.main_texture), &self.device, &self.queue);

        let texture_bind_group_layout =
            self.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("texture_bind_group_layout"),
                });

        let diffuse_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        self.gpu_manager
            .add_bind_group_layout("texture", texture_bind_group_layout);

        self.gpu_manager
            .add_bind_group("texture", diffuse_bind_group);

        material
            .textures
            .insert(material.main_texture.clone(), texture);
    }
}

fn get_texture(
    path: PathBuf,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> super::texture::Texture {
    let buffer = std::fs::read(&path).unwrap();

    super::texture::Texture::new(device, queue, buffer)
}
