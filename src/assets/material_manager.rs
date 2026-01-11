use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use cgmath::{
    Array,
    num_traits::{one},
};
use wgpu::{
    TextureFormat::{Rgba8Unorm, Rgba8UnormSrgb},
    util::DeviceExt,
};

use crate::{
    assets::texture_manager::TextureManager,
    math::*,
    renderer::{
        gpu_manager::{GpuManager, LayoutKind},
        uniform::MaterialUniform,
    },
};

pub type MaterialId = PathBuf;

pub const MATERIAL_TEXTURE_COUNT: usize = 5;
pub const MATERIAL_TEXTURE_SLOTS: [MaterialTextureSlot; MATERIAL_TEXTURE_COUNT] = [
    MaterialTextureSlot::BaseColor,
    MaterialTextureSlot::Normal,
    MaterialTextureSlot::MetallicRoughness,
    MaterialTextureSlot::Emissive,
    MaterialTextureSlot::Occlusion,
];

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum MaterialTextureSlot {
    BaseColor = 0,
    Normal = 1,
    MetallicRoughness = 2,
    Emissive = 3,
    Occlusion = 4,
}

impl MaterialTextureSlot {
    pub fn color_space(self) -> wgpu::TextureFormat {
        match self {
            MaterialTextureSlot::BaseColor | MaterialTextureSlot::Emissive => Rgba8UnormSrgb,

            MaterialTextureSlot::Normal
            | MaterialTextureSlot::MetallicRoughness
            | MaterialTextureSlot::Occlusion => Rgba8Unorm,
        }
    }
}

#[derive(Clone)]
pub struct MaterialPBR {
    pub name: String,

    texture_slot: [Option<PathBuf>; MATERIAL_TEXTURE_COUNT],

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

            texture_slot: [const { None }; MATERIAL_TEXTURE_COUNT],

            base_color_factor: Vec4::from_value(one()),
            emissive_factor: Vec4::from_value(one()),
            roughness_factor: one(),
            metallic_factor: one(),
            normal_scale: one(),
            occlusion_strength: one(),
            use_color_texture: false,
            use_metal_roughness_texture: false,
            use_normal_texture: false,
            use_emissive_texture: false,
            use_occlusion_texture: false,
        }
    }
}

impl MaterialPBR {
    pub fn set_path(&mut self, slot: MaterialTextureSlot, path: Option<PathBuf>) {
        self.texture_slot[slot as usize] = path;
    }
    pub fn some_or_fallback(&self, slot: MaterialTextureSlot) -> &Path {
        self.texture_slot[slot as usize]
            .as_deref()
            .unwrap_or_else(|| Path::new(""))
    }

    pub fn get_path(&self, slot: MaterialTextureSlot) -> Option<&Path> {
        self.texture_slot[slot as usize].as_deref()
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
        gpu_manager: &GpuManager,
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
    materials: HashMap<MaterialId, Material>,
    default: Material,
}

impl MaterialManager {
    pub fn new(
        device: &wgpu::Device,
        gpu_manager: &GpuManager,
        texture_manager: &mut TextureManager,
    ) -> Self {
        let default = Material::create_default(&device, texture_manager, &gpu_manager);
        Self {
            materials: HashMap::new(),
            default,
        }
    }

    pub fn create(
        &mut self,
        device: &wgpu::Device,
        gpu_manager: &GpuManager,
        texture_manager: &mut TextureManager,
        material_pbr: &MaterialPBR,
    ) -> MaterialId {
        // create also textures
        let uniform_buffer = create_uniform(device, &material_pbr);
        let bind_group = create_bindgroup(
            device,
            &material_pbr,
            &uniform_buffer,
            texture_manager,
            gpu_manager,
        );

        let material = Material {
            material_pbr: material_pbr.clone(),
            bind_group,
            uniform_buffer,
        };

        let material_id = PathBuf::from(material_pbr.name.clone());

        self.materials.insert(material_id.clone(), material);

        material_id
    }

    pub fn get(&self, id: &MaterialId) -> &Material {
        self.materials.get(id).unwrap_or(&self.default)
    }

    pub fn get_mut(&mut self, id: &MaterialId) -> &mut Material {
        self.materials.get_mut(id).unwrap_or(&mut self.default)
    }
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
    texture_mgr: &mut TextureManager,
    gpu_manager: &GpuManager,
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

    use MaterialTextureSlot::*;

    let base = texture_mgr.create_texture(
        material_pbr.some_or_fallback(BaseColor),
        BaseColor.color_space(),
    );
    let met_rough = texture_mgr.create_texture(
        material_pbr.some_or_fallback(MetallicRoughness),
        MetallicRoughness.color_space(),
    );
    let normal =
        texture_mgr.create_texture(material_pbr.some_or_fallback(Normal), Normal.color_space());
    let emissive = texture_mgr.create_texture(
        material_pbr.some_or_fallback(Emissive),
        Emissive.color_space(),
    );
    let occlusion = texture_mgr.create_texture(
        material_pbr.some_or_fallback(Occlusion),
        Occlusion.color_space(),
    );

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
                resource: wgpu::BindingResource::TextureView(&base.view),
            },
            // normal texture
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&normal.view),
            },
            // metallic_roughness texture
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&met_rough.view),
            },
            // material emissive
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(&emissive.view),
            },
            // material occlusion
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::TextureView(&occlusion.view),
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
