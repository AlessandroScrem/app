use super::*;
use std::path::{Path, PathBuf};

use wgpu::TextureFormat::{Rgba8Unorm, Rgba8UnormSrgb};

use crate::{math::*, renderer::uniform::MaterialUniform};


#[derive(Default, Hash, Eq, PartialEq, Clone)]
pub enum ShaderId {
    #[default]
    Pbr,
}

#[derive(Default, Hash, Eq, PartialEq, Clone)]
pub struct MaterialKey {
    pub shader: ShaderId,
    pub textures: [Option<TextureId>; MATERIAL_TEXTURE_COUNT],
}

#[derive(Clone)]
pub struct MaterialDesc {
    pub key: MaterialKey,
    pub use_texture_slot: [bool; MATERIAL_TEXTURE_COUNT],

    pub base_color_factor: Vec4,
    pub emissive_factor: Vec4,
    pub roughness_factor: f32,
    pub metallic_factor: f32,
    pub normal_scale: f32,
    pub occlusion_strength: f32,
}

impl MaterialDesc {
    pub fn get_texture_slot(&self, slot: MaterialTextureSlot) -> Option<TextureId> {
        self.key.textures.get(slot as usize).copied().flatten()
    }
}

#[derive(Default)]
pub struct MaterialAssets {
    storage: SlotMap<MaterialId, MaterialDesc>,
    lookup: HashMap<MaterialKey, MaterialId>,
}

impl MaterialAssets {
    pub fn get_or_create(
        &mut self,
        key: MaterialKey,
        desc_fn: impl FnOnce() -> MaterialDesc,
    ) -> MaterialId {
        if let Some(id) = self.lookup.get(&key) {
            return *id;
        }

        let desc = desc_fn();
        let id = self.storage.insert(desc);
        self.lookup.insert(key, id);
        id
    }

    pub fn get(&self, id: MaterialId) -> Option<&MaterialDesc> {
        self.storage.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (MaterialId, &MaterialDesc)> {
        self.storage.iter()
    }
}


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

impl MaterialTextureSlot {
    pub const ALL: [MaterialTextureSlot; 5] = [
        MaterialTextureSlot::BaseColor,
        MaterialTextureSlot::Normal,
        MaterialTextureSlot::MetallicRoughness,
        MaterialTextureSlot::Emissive,
        MaterialTextureSlot::Occlusion,
    ];
}

#[derive(Clone, Debug)]
pub struct MaterialPBR {
    pub name: String,

    texture_slot: [Option<PathBuf>; MATERIAL_TEXTURE_COUNT],
    pub use_texture_slot: [bool; MATERIAL_TEXTURE_COUNT],

    pub base_color_factor: Vec4,
    pub emissive_factor: Vec4,
    pub roughness_factor: f32,
    pub metallic_factor: f32,
    pub normal_scale: f32,
    pub occlusion_strength: f32,
}
impl Default for MaterialPBR {
    fn default() -> Self {
        Self {
            name: "Default".into(),

            texture_slot: [const { None }; MATERIAL_TEXTURE_COUNT],
            use_texture_slot: [const { false }; MATERIAL_TEXTURE_COUNT],

            base_color_factor: Vec4::from_value(one()),
            emissive_factor: Vec4::from_value(zero()),
            roughness_factor: one(),
            metallic_factor: one(),
            normal_scale: one(),
            occlusion_strength: one(),
        }
    }
}

impl PartialEq for MaterialPBR {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.texture_slot == other.texture_slot
            && self.use_texture_slot == other.use_texture_slot
            && self.base_color_factor == other.base_color_factor
            && self.emissive_factor == other.emissive_factor
            && self.roughness_factor == other.roughness_factor
            && self.metallic_factor == other.metallic_factor
            && self.normal_scale == other.normal_scale
            && self.occlusion_strength == other.occlusion_strength
    }
}

impl MaterialPBR {
    pub fn set_path(&mut self, slot: MaterialTextureSlot, path: Option<PathBuf>) {
        self.texture_slot[slot as usize] = path;
        self.use_texture_slot[slot as usize] = true;
    }
    pub fn some_or_fallback(&self, slot: MaterialTextureSlot) -> &Path {
        self.texture_slot[slot as usize]
            .as_deref()
            .unwrap_or_else(|| Path::new(""))
    }

    pub fn get_path(&self, slot: MaterialTextureSlot) -> Option<&Path> {
        self.texture_slot[slot as usize].as_deref()
    }
    pub fn get_used_texture_slot(&self, slot: MaterialTextureSlot) -> bool {
        self.use_texture_slot[slot as usize]
    }
    pub fn set_used_texture_slot(&mut self, slot: MaterialTextureSlot, flag: bool) {
        self.use_texture_slot[slot as usize] = flag
    }
}

impl From<&MaterialDesc> for MaterialUniform {
    fn from(value: &MaterialDesc) -> Self {
        Self {
            color_factor: value.base_color_factor.into(),
            emissive_factor: value.emissive_factor.into(),
            metallic_factor: value.metallic_factor,
            roughness_factor: value.roughness_factor,
            normal_scale: value.normal_scale,
            occlusion_strength: value.occlusion_strength,
            use_color_texture: value.use_texture_slot[MaterialTextureSlot::BaseColor as usize]
                as u32,
            use_normal_texture: value.use_texture_slot[MaterialTextureSlot::Normal as usize] as u32,
            use_metal_roughness_texture: value.use_texture_slot
                [MaterialTextureSlot::MetallicRoughness as usize]
                as u32,
            use_emissive_texture: value.use_texture_slot[MaterialTextureSlot::Emissive as usize]
                as u32,
            use_occlusion_texture: value.use_texture_slot[MaterialTextureSlot::Occlusion as usize]
                as u32,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn should_create_material_from_id() {
    }
}
