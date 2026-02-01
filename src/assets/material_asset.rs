use super::*;
use std::path::PathBuf;
use texture_asset::ColorSpace;
use crate::{math::*, renderer::uniform::MaterialUniform};

#[derive(Default, Hash, Eq, PartialEq, Clone)]
pub enum ShaderId {
    #[default]
    Pbr,
}

pub const MATERIAL_TEXTURE_COUNT: usize = 5;
pub const MATERIAL_TEXTURE_SLOTS: [MaterialTextureSlot; MATERIAL_TEXTURE_COUNT] = [
    MaterialTextureSlot::BaseColor,
    MaterialTextureSlot::Normal,
    MaterialTextureSlot::MetallicRoughness,
    MaterialTextureSlot::Emissive,
    MaterialTextureSlot::Occlusion,
];

#[derive(Default, Hash, Eq, PartialEq, Clone)]
pub struct MaterialKey {
    pub name: String,
    pub shader: ShaderId,
    pub textures: [Option<TextureId>; MATERIAL_TEXTURE_COUNT],
}


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
    pub fn color_space(self) -> ColorSpace {
        match self {
            MaterialTextureSlot::BaseColor | MaterialTextureSlot::Emissive => ColorSpace::Srgba8,

            MaterialTextureSlot::Normal
            | MaterialTextureSlot::MetallicRoughness
            | MaterialTextureSlot::Occlusion => ColorSpace::Rgba8,
        }
    }
}

impl MaterialTextureSlot {
    pub const ALL: [MaterialTextureSlot; MATERIAL_TEXTURE_COUNT] = [
        MaterialTextureSlot::BaseColor,
        MaterialTextureSlot::Normal,
        MaterialTextureSlot::MetallicRoughness,
        MaterialTextureSlot::Emissive,
        MaterialTextureSlot::Occlusion,
    ];
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

impl Default for MaterialDesc {
    fn default() -> Self {
        MaterialDesc {
            key: MaterialKey::default(),
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

impl MaterialDesc {
    pub fn get_texture_slot(&self, slot: MaterialTextureSlot) -> Option<TextureId> {
        self.key.textures.get(slot as usize).copied().flatten()
    }

    pub fn set_name(&mut self, name: &str) {
        self.key.name = name.into();
    }

    pub fn set_texture(&mut self, texture_asset: &mut TextureAssets, slot: MaterialTextureSlot, path: Option<PathBuf>) {
        if let Some(path) = path  {
            let key = TextureKey::File {
                color_space: slot.color_space().into(),
                path: path.into(),
                usage: slot.into(),
            };
            let desc = super::TextureDesc::File {
                key,
                sampler: super::SamplerDesc::Linear,
                mipmaps: false,
            };
            let id = texture_asset.get_or_create(desc);
            self.key.textures[slot as usize] = Some(id);
            self.use_texture_slot[slot as usize] = true;
        }

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
    fn should_create_material_from_id() {}
}
