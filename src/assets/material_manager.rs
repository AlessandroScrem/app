use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{assets::texture::Texture, resources::gpu_manager::GPUResourceManager};

#[derive(PartialEq, Eq, Hash)]
pub enum TextureType {
    Main,
    Normal,
    Roughness,
}

pub struct TextureBinding {
    pub name: String,
    texture: Texture,
}

pub struct Material {
    pub roughness: f32,
    pub metallic: f32,
    pub roughness_override: f32,
    pub metallic_override: f32,
    pub color: cgmath::Vector4<f32>,
    pub textures: HashMap<TextureType, TextureBinding>,
    pub bind_group: Option<wgpu::BindGroup>,
}

pub struct MaterialManager {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    gpu_manager: Arc<GPUResourceManager>,
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
        }
    }

    pub fn create_material(
        &mut self,
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
        let main_texture = get_texture_url(&main_info, &images).unwrap_or("white.png".to_string());
        let normal_texture = normal_texture.unwrap_or("white.png".to_string());
        let roughness_texture = roughness_texture.unwrap_or("white.png".to_string());

        let mut textures: HashMap<TextureType, TextureBinding> = HashMap::new();

        let main_texture = load_texture(
            &main_texture,
            parent_path,
            &self.device,
            &self.queue,
            true,
        );
        let normal_texture = load_texture(
            &normal_texture,
            parent_path,
            &self.device,
            &self.queue,
            true,
        );
        let roughness_texture = load_texture(
            &roughness_texture,
            parent_path,
            &self.device,
            &self.queue,
            true,
        );

        
        
        let texture_bind_group_layout = self
            .gpu_manager
            .get_layout("texture")
            .expect("unable to find bind group layout");
        
        let diffuse_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&main_texture.texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&normal_texture.texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&roughness_texture.texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&roughness_texture.texture.sampler),
                },
                ],
                label: Some("texture_bind_group"),
            });
            
            textures.insert(
                TextureType::Main,
                main_texture,
            );
            textures.insert(
                TextureType::Normal,
                normal_texture,
            );
            textures.insert(
                TextureType::Roughness,
                roughness_texture,
            );

        Material {
            roughness,
            metallic,
            roughness_override: if has_pbr_texture { 0.0 } else { 1.0 },
            metallic_override: if has_pbr_texture { 0.0 } else { 1.0 },
            color,
            textures,
            bind_group:  Some(diffuse_bind_group),
        }
    }
}

fn load_texture(
    name: &str,
    path: &Path,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    is_normal: bool,
) -> TextureBinding {
    let filepath = {
        let candidate = path.join(name);
        candidate
            .is_file()
            .then(|| candidate)
            .unwrap_or_else(|| PathBuf::from("assets/core/white.png"))
    };
    let buffer =
        std::fs::read(&filepath).expect(&format!("Impossibile leggere il file {:?}", filepath));

    let texture = Texture::new(device, queue, buffer, is_normal);

    TextureBinding {
        name: name.to_string(),
        texture,
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
