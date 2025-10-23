use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use wgpu::{TextureFormat, util::DeviceExt};

use crate::{
    prelude::*,
    assets::texture_manager::TextureManager,
    renderer::{
        gpu_manager::{GPUResourceManager, LayoutKind},
        uniform::MaterialUniform,
    },
    math::*,
};

pub struct Material {
    pub main_texture: PathBuf,
    pub normal_texture: PathBuf,
    pub metallic_roughness_texture: PathBuf,

    pub roughness: f32,
    pub metallic: f32,
    pub roughness_use_texture: u32,
    pub metallic_use_texture: u32,
    pub color_use_texture: u32,
    pub color: Vec4,
    pub bind_group: Option<wgpu::BindGroup>,
    pub material_uniform_buffer: Option<wgpu::Buffer>,
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
        let color = vec4(
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

        let metallic_roughness_texture = get_texture_url(&roughness_info, &images);

        let roughness_use_texture: u32 = metallic_roughness_texture.is_some() as u32;
        let metallic_use_texture: u32 = metallic_roughness_texture.is_some() as u32;
        let color_use_texture = pbr.base_color_texture().is_some() as u32;

        let main_texture = get_texture_url(&main_info, &images)
            .map(|s| parent_path.join(s))
            .unwrap_or("no-name".into());
        let normal_texture = normal_texture
            .map(|s| parent_path.join(s))
            .unwrap_or("no-name".into());
        let metallic_roughness_texture = metallic_roughness_texture
            .map(|s| parent_path.join(s))
            .unwrap_or("no-name".into());

        let timer = std::time::Instant::now();
        let bind0 = texture_manager.get_or_create(&main_texture, TextureFormat::Rgba8UnormSrgb);
        let bind1 = texture_manager.get_or_create(&normal_texture, TextureFormat::Rgba8Unorm);
        let bind2 =
            texture_manager.get_or_create(&metallic_roughness_texture, TextureFormat::Rgba8Unorm);

        info!(
            "--\t Load matererial textures took {} ms",
            timer.elapsed().as_millis()
        );

        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let material_uniform_buffer =
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Material Uniform Buffer"),
                    contents: bytemuck::cast_slice(&[MaterialUniform {
                        color: [color.x, color.y, color.z, color.w],
                        metallic,
                        roughness,
                        roughness_use_texture,
                        metallic_use_texture,
                        color_use_texture,
                        ..Default::default()
                    }]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let texture_bind_group_layout = self.gpu_manager.get_layout(LayoutKind::Material);

        let diffuse_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                // main texture
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&bind0.view),
                },
                // normal texture
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&bind1.view),
                },
                // metallic_roughness texture
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&bind2.view),
                },
                // material uniform
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: material_uniform_buffer.as_entire_binding(),
                },
            ],
            label: Some("texture_bind_group"),
        });

        Material {
            main_texture,
            normal_texture,
            metallic_roughness_texture,
            roughness,
            metallic,
            roughness_use_texture,
            metallic_use_texture,
            color_use_texture,
            color,
            bind_group: Some(diffuse_bind_group),
            material_uniform_buffer: Some(material_uniform_buffer),
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
