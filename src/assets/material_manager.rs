use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use cgmath::{Array, num_traits::{one, zero}};
use log::info;
use wgpu::{
    TextureFormat::{Rgba8Unorm, Rgba8UnormSrgb},
    util::DeviceExt,
};

use crate::{
    assets::texture_manager::TextureManager,
    math::*,
    renderer::{
        gpu_manager::{GPUResourceManager, LayoutKind},
        uniform::MaterialUniform,
    },
};

pub type MaterialId = PathBuf;

pub struct MaterialPBR {
    pub name: String,

    pub base_texture_path: PathBuf,
    pub normal_texture_path: PathBuf,
    pub met_rough_texture_path: PathBuf,
    pub emissive_texture_path: PathBuf,
    pub occlusion_texture_path: PathBuf,

    pub base_color_factor: Vec4,
    pub emissive_factor: Vec4,
    pub roughness_factor: f32,
    pub metallic_factor: f32,
    pub normal_scale: f32,
    pub occlusion_strength: f32,
    pub use_color_texture: bool,
    pub use_metal_roughness_texture: bool,
    pub use_normal_texture: bool,
    pub use_emissive_texture: bool,
    pub use_occlusion_texture: bool,
}
impl Default for MaterialPBR {
    fn default() -> Self {
        Self {
            name: "Default".into(),

            base_texture_path: "void".into(),
            normal_texture_path: "void".into(),
            met_rough_texture_path: "void".into(),
            emissive_texture_path: "void".into(),
            occlusion_texture_path: "void".into(),

            base_color_factor: Vec4::from_value(one()),
            emissive_factor: Vec4::from_value(one()),
            roughness_factor: one(),
            metallic_factor: one(),
            normal_scale: one(),
            occlusion_strength: zero(),
            use_color_texture: false,
            use_metal_roughness_texture: false,
            use_normal_texture: false,
            use_emissive_texture: false,
            use_occlusion_texture: false,
        }
    }
}

impl From<&MaterialPBR> for MaterialUniform {
    fn from(value: &MaterialPBR) -> Self {
        Self {
            color_factor: value.base_color_factor.into(),
            emissive_factor: value.emissive_factor.into(),
            metallic_factor: value.metallic_factor,
            roughness_factor: value.roughness_factor,
            normal_scale: value.normal_scale,
            occlusion_strength: value.occlusion_strength,
            use_color_texture: value.use_color_texture as u32,
            use_metal_roughness_texture: value.use_metal_roughness_texture as u32,
            use_normal_texture: value.use_normal_texture as u32,
            use_emissive_texture: value.use_emissive_texture as u32,
            use_occlusion_texture: value.use_occlusion_texture as u32,
            ..Default::default()
        }
    }
}

pub struct Material {
    pub material_pbr: MaterialPBR,
    pub bind_group: wgpu::BindGroup,
    pub uniform_buffer: wgpu::Buffer,
}

impl Material {
    fn create_default(
        device: &wgpu::Device,
        texture_manager: &mut TextureManager,
        gpu_manager: &GPUResourceManager,
    ) -> Self {
        let material_pbr = MaterialPBR::default();
        let uniform_buffer = create_uniform(device, &material_pbr);
        let bind_group = create_bindgroup(
            device,
            &material_pbr,
            &uniform_buffer,
            texture_manager,
            gpu_manager,
        );
        Self {
            material_pbr,
            bind_group,
            uniform_buffer,
        }
    }
}

pub struct MaterialManager {
    device: Arc<wgpu::Device>,
    gpu_manager: Arc<GPUResourceManager>,
    materials: HashMap<MaterialId, Material>,
    default: Material,
}

impl MaterialManager {
    pub fn new(
        device: Arc<wgpu::Device>,
        gpu_manager: Arc<GPUResourceManager>,
        texture_manager: &mut TextureManager,
    ) -> Self {
        let default = Material::create_default(&device, texture_manager, &gpu_manager);
        Self {
            device,
            gpu_manager,
            materials: HashMap::new(),
            default,
        }
    }

    pub fn get(&self, id: &MaterialId) -> &Material {
        self.materials.get(id).unwrap_or(&self.default)
    }

    pub fn get_mut(&mut self, id: &MaterialId) -> &mut Material {
        self.materials.get_mut(id).unwrap_or(&mut self.default)
    }

    pub fn create_material(
        &mut self,
        texture_manager: &mut TextureManager,
        gltf_material: &gltf::Material,
        images: &Vec<gltf::Image<'_>>,
        path: PathBuf,
    ) -> MaterialId {
        let name = gltf_material.name().unwrap_or("material_no_name");
        let material_id = path.join(&name);

        if self.materials.contains_key(&material_id) {
            return material_id;
        }
        let parent_path = path.parent().expect("Unable to find parent path");

        // pbr materials
        let pbr = gltf_material.pbr_metallic_roughness();
        let color_factor = pbr.base_color_factor();
        let roughness_factor = pbr.roughness_factor();
        let metallic_factor = pbr.metallic_factor();
        let emissive_factor = gltf_material.emissive_factor();

        let use_color_texture = pbr.base_color_texture().is_some();
        let use_metal_roughness_texture = pbr.base_color_texture().is_some();
        let use_normal_texture = gltf_material.normal_texture().is_some();
        let use_emissive_texture = gltf_material.emissive_texture().is_some();
        let use_occlusion_texture = gltf_material.occlusion_texture().is_some();

        let base_texture_path = get_texture_url(pbr.base_color_texture(), parent_path, &images);
        let met_rough_texture =
            get_texture_url(pbr.metallic_roughness_texture(), parent_path, &images);
        let emissive_texture_path =
            get_texture_url(gltf_material.emissive_texture(), parent_path, &images);

        let normal_texture_path = gltf_material
            .normal_texture()
            .map(|nt| nt.texture().source().source())
            .and_then(|s| {
                if let gltf::image::Source::Uri { uri, .. } = s {
                    Some(parent_path.join(uri))
                } else {
                    None
                }
            });

        let normal_scale = gltf_material
            .normal_texture()
            .map(|nt| nt.scale())
            .unwrap_or(1.0);

        let occlusion_texture_path = gltf_material
            .occlusion_texture()
            .map(|ot| ot.texture().source().source())
            .and_then(|s| {
                if let gltf::image::Source::Uri { uri, .. } = s {
                    Some(parent_path.join(uri))
                } else {
                    None
                }
            });

        let occlusion_strength = gltf_material
            .occlusion_texture()
            .map(|ot| ot.strength())
            .unwrap_or(1.0);

        let material_pbr = MaterialPBR {
            name: name.into(),
            base_color_factor: color_factor.into(),
            emissive_factor: Vec3::from(emissive_factor).extend(1.0),
            base_texture_path: base_texture_path.unwrap_or_default(),
            normal_texture_path: normal_texture_path.unwrap_or_default(),
            met_rough_texture_path: met_rough_texture.unwrap_or_default(),
            emissive_texture_path: emissive_texture_path.unwrap_or_default(),
            occlusion_texture_path: occlusion_texture_path.unwrap_or_default(),
            roughness_factor,
            metallic_factor,
            normal_scale,
            occlusion_strength,
            use_color_texture,
            use_metal_roughness_texture,
            use_normal_texture,
            use_emissive_texture,
            use_occlusion_texture
        };

        let timer = std::time::Instant::now();
        // create also textures
        let uniform_buffer = create_uniform(&self.device, &material_pbr);
        let bind_group = create_bindgroup(
            &self.device,
            &material_pbr,
            &uniform_buffer,
            texture_manager,
            &self.gpu_manager,
        );

        info!(
            "--\t Load matererial {} textures took {} ms",
            material_id.to_string_lossy(),
            timer.elapsed().as_millis()
        );

        let material = Material {
            material_pbr,
            bind_group,
            uniform_buffer,
        };

        self.materials.insert(material_id.clone(), material);
        material_id
    }
}

fn get_texture_url(
    info: Option<gltf::texture::Info<'_>>,
    path: &Path,
    images: &[gltf::Image<'_>],
) -> Option<PathBuf> {
    let info = info?;
    let image = images.get(info.texture().index())?;

    if let gltf::image::Source::Uri { uri, .. } = image.source() {
        return Path::new(uri).file_name().map(|u| path.join(u));
    }

    None
}

fn create_uniform(device: &wgpu::Device, material_pbr: &MaterialPBR) -> wgpu::Buffer {
    let uniform = MaterialUniform::from(material_pbr);

    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Material Uniform Buffer"),
        contents: bytemuck::bytes_of(&uniform),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    uniform_buffer
}

fn create_bindgroup(
    device: &wgpu::Device,
    material_pbr: &MaterialPBR,
    uniform_buffer: &wgpu::Buffer,
    texture_manager: &mut TextureManager,
    gpu_manager: &GPUResourceManager,
) -> wgpu::BindGroup {
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let base_texture =
        texture_manager.get_or_create(&material_pbr.base_texture_path, Rgba8UnormSrgb);
    let met_rough_texture =
        texture_manager.get_or_create(&material_pbr.met_rough_texture_path, Rgba8Unorm);
    let normal_texture =
        texture_manager.get_or_create(&material_pbr.normal_texture_path, Rgba8Unorm);
    let emissive_texture =
        texture_manager.get_or_create(&material_pbr.emissive_texture_path, Rgba8Unorm);
    let occlusion_texture =
        texture_manager.get_or_create(&material_pbr.occlusion_texture_path, Rgba8Unorm);

    let texture_bind_group_layout = gpu_manager.get_layout(LayoutKind::Material);

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &texture_bind_group_layout,
        label: Some("Material  bind_group"),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            // main texture
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&base_texture.view),
            },
            // normal texture
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&normal_texture.view),
            },
            // metallic_roughness texture
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&met_rough_texture.view),
            },
            // material emissive
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(&emissive_texture.view),
            },
            // material occlusion
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::TextureView(&occlusion_texture.view),
            },
            // uniform buffer
            wgpu::BindGroupEntry {
                binding: 6,
                resource: uniform_buffer.as_entire_binding(),
            },
        ],
    });
    bind_group
}
