use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

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
    _materials: HashMap<PathBuf, Material>,
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
            _materials: HashMap::new(),
        }
    }

    pub fn add_material(&mut self, mut material: Material, path: PathBuf) {
        let parent_path = path.parent().expect("anable to find parent path");

        let main_texture = load_texture(
            &material.main_texture,
            parent_path,
            &self.device,
            &self.queue,
            false,
        );
        let normal_texture = load_texture(
            &material.normal_texture,
            parent_path,
            &self.device,
            &self.queue,
            true,
        );

        // let roughness_texture = load_texture(
        //     &material.roughness_texture,
        //     parent_path,
        //     &self.device,
        //     &self.queue,
        // );

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
                        // normal map
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
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
                    resource: wgpu::BindingResource::TextureView(&main_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&main_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
                },
            ],
            label: Some("texture_bind_group"),
        });

        self.gpu_manager
            .add_bind_group_layout("texture", texture_bind_group_layout);

        self.gpu_manager
            .add_bind_group("texture", diffuse_bind_group);

        material
            .textures
            .insert(material.main_texture.clone(), main_texture);
    }
}

fn load_texture(
    name: &str,
    path: &Path,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    is_normal: bool,
) -> super::texture::Texture {
    let filepath = {
        let candidate = path.join(name);
        candidate
            .is_file()
            .then(|| candidate)
            .unwrap_or_else(|| PathBuf::from("assets/core/white.png"))
    };
    let buffer =
        std::fs::read(&filepath).expect(&format!("Impossibile leggere il file {:?}", filepath));

    super::texture::Texture::new(device, queue, buffer, is_normal)
}
