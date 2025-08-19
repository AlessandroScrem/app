use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{assets::texture_manager::TextureManager, resources::gpu_manager::GPUResourceManager};

pub struct Material {
    pub main_texture: PathBuf,
    pub normal_texture: PathBuf,
    pub roughness_texture: PathBuf,

    pub roughness: f32,
    pub metallic: f32,
    pub roughness_override: f32,
    pub metallic_override: f32,
    pub color: cgmath::Vector4<f32>,
    pub bind_group: Option<wgpu::BindGroup>,
}

pub struct MaterialManager {
    device: Arc<wgpu::Device>,
    gpu_manager: Arc<GPUResourceManager>,
}

impl MaterialManager {
    pub fn new(device: Arc<wgpu::Device>, gpu_manager: Arc<GPUResourceManager>) -> Self {
        Self {
            device,
            gpu_manager,
        }
    }

    pub fn create_material(
        &mut self,
        texture_manager: &mut TextureManager,
        gltf_material: &gltf::Material,
        images: &Vec<gltf::Image<'_>>,
        path: PathBuf,
    ) -> Material {
        let parent_path = path.parent().expect("anable to find parent path");

        // materials
        let pbr = gltf_material.pbr_metallic_roughness();
        let color_factor = pbr.base_color_factor();
        let color = cgmath::Vector4::new(
            color_factor[0],
            color_factor[1],
            color_factor[2],
            color_factor[3],
        );

        let mut normal_texture = None;
        let normals_texture = gltf_material.normal_texture();
        if normals_texture.is_some() {
            let normal_source = normals_texture.unwrap().texture().source().source();
            match normal_source {
                gltf::image::Source::Uri { uri, .. } => {
                    let texture_file_name = Some(
                        Path::new(&uri)
                            .file_name()
                            .and_then(std::ffi::OsStr::to_str)
                            .unwrap()
                            .to_string(),
                    );
                    if texture_file_name.is_some() {
                        normal_texture = Some(texture_file_name.unwrap());
                    }
                }
                _ => (),
            }
        }

        let main_info = pbr.base_color_texture();
        let roughness_info = pbr.metallic_roughness_texture();
        let roughness = pbr.roughness_factor();
        let metallic = pbr.metallic_factor();

        let roughness_texture = get_texture_url(&roughness_info, &images);
        let has_pbr_texture = roughness_texture.is_some();

        let main_texture = get_texture_url(&main_info, &images)
            .map(|s| parent_path.join(s))
            .unwrap_or("no-name".into());
        let normal_texture = normal_texture
            .map(|s| parent_path.join(s))
            .unwrap_or("no-name".into());
        let roughness_texture = roughness_texture
            .map(|s| parent_path.join(s))
            .unwrap_or("no-name".into());

        let bind0 = texture_manager.get_or_create(&main_texture, false);
        let bind1 = texture_manager.get_or_create(&normal_texture, true);
        let bind2 = texture_manager.get_or_create(&roughness_texture, false);

        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let texture_bind_group_layout = self.gpu_manager.get_layout("texture");

        let diffuse_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&bind0.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&bind1.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&bind2.view),
                },
            ],
            label: Some("texture_bind_group"),
        });

        Material {
            main_texture,
            normal_texture,
            roughness_texture,
            roughness,
            metallic,
            roughness_override: if has_pbr_texture { 0.0 } else { 1.0 },
            metallic_override: if has_pbr_texture { 0.0 } else { 1.0 },
            color,
            bind_group: Some(diffuse_bind_group),
        }
    }
}

fn get_texture_url(
    info: &Option<gltf::texture::Info<'_>>,
    images: &Vec<gltf::Image<'_>>,
) -> Option<String> {
    let mut file_name = None;
    if info.is_some() {
        let info = info.as_ref().unwrap();
        let tex = info.texture();

        let image: Option<&gltf::Image<'_>> = images.get(tex.index());
        if image.is_some() {
            let image = image.unwrap();
            let source = image.source();
            match source {
                gltf::image::Source::Uri { uri, .. } => {
                    let texture_file_name = Some(Path::new(&uri).to_str().unwrap().to_string());
                    if texture_file_name.is_some() {
                        file_name = Some(texture_file_name.unwrap());
                    }
                }
                _ => (),
            }
        }
    }
    file_name
}
