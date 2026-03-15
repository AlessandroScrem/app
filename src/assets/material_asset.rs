use super::*;
use crate::{math::*, renderer::uniform::MaterialUniform};
use cgmath::AbsDiffEq;
use std::ops::{Index, IndexMut};
use std::{cell::Cell, path::PathBuf};
use texture_asset::ColorSpace;

#[derive(Debug, Default, Hash, Eq, PartialEq, Clone)]
pub enum ShaderId {
    #[default]
    Pbr,
}

pub const MATERIAL_TEXTURE_COUNT: usize = 5;

#[derive(Debug, Default, Hash, Eq, PartialEq, Clone)]
pub(crate) struct TestureSet {
    textures: [Option<TextureId>; MATERIAL_TEXTURE_COUNT],
}

impl Index<MaterialTextureSlot> for TestureSet {
    type Output = Option<TextureId>;

    fn index(&self, slot: MaterialTextureSlot) -> &Self::Output {
        &self.textures[slot as usize]
    }
}

impl IndexMut<MaterialTextureSlot> for TestureSet {
    fn index_mut(&mut self, slot: MaterialTextureSlot) -> &mut Self::Output {
        &mut self.textures[slot as usize]
    }
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
    name: String,
    #[allow(unused)]
    shader: ShaderId,

    pub texture_set: TestureSet,
    use_texture_slot: [bool; MATERIAL_TEXTURE_COUNT],

    pub base_color_factor: Vec4,
    pub emissive_factor: Vec4,
    pub roughness_factor: f32,
    pub metallic_factor: f32,
    pub normal_scale: f32,
    pub occlusion_strength: f32,
}

// PartialEq ignore: name , shader
impl PartialEq for MaterialDesc {
    fn eq(&self, other: &Self) -> bool {
        self.texture_set.textures == other.texture_set.textures
            && self.use_texture_slot == other.use_texture_slot
            && self
                .base_color_factor
                .abs_diff_eq(&other.base_color_factor, Default::default())
            && self
                .emissive_factor
                .abs_diff_eq(&other.emissive_factor, Default::default())
            && self.roughness_factor.to_bits() == other.roughness_factor.to_bits()
            && self.metallic_factor.to_bits() == other.metallic_factor.to_bits()
            && self.normal_scale.to_bits() == other.normal_scale.to_bits()
            && self.occlusion_strength.to_bits() == other.occlusion_strength.to_bits()
    }
}

impl Default for MaterialDesc {
    fn default() -> Self {
        MaterialDesc {
            texture_set: TestureSet::default(),
            name: String::new(),
            shader: ShaderId::default(),

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
        self.texture_set
            .textures
            .get(slot as usize)
            .copied()
            .flatten()
    }
    fn estimated_size() -> usize {
        size_of::<MaterialDesc>()
    }

    pub fn slot_get(&self, slot: MaterialTextureSlot) -> bool {
        self.use_texture_slot[slot as usize]
    }
    pub fn slot_set(&mut self, slot: MaterialTextureSlot, flag: bool) {
        self.use_texture_slot[slot as usize] = flag;
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn set_name(&mut self, name: &str) {
        self.name = name.into();
    }

    pub fn set_texture(
        &mut self,
        texture_asset: &mut TextureAssets,
        slot: MaterialTextureSlot,
        path: Option<PathBuf>,
    ) {
        if let Some(path) = path {
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
            self.texture_set.textures[slot as usize] = Some(id);
            self.use_texture_slot[slot as usize] = true;
        }
    }
}

#[derive(Clone)]
struct MaterialAsset {
    desc: MaterialDesc,
    ref_count: Cell<u32>,
}

impl HasStats for MaterialAssets {
    fn get_stats(&self) -> ResourceStats {
        self.stats.clone()
    }
}

#[derive(Default)]
pub struct MaterialAssets {
    storage: SlotMap<MaterialId, MaterialAsset>,
    stats: ResourceStats,
}

impl MaterialAssets {
    pub fn get_or_create(&mut self, desc: MaterialDesc) -> MaterialId {
        match self.find_duplicate(&desc) {
            Some(id) => {
                let mat = &self.storage[id];
                mat.ref_count.set(mat.ref_count.get() + 1);
                self.stats.add_shared();
                id
            }
            None => {
                let id = self.storage.insert(MaterialAsset {
                    desc: desc.clone(),
                    ref_count: Cell::new(1),
                });
                if self.storage[id].desc.name.is_empty() {
                    self.storage[id].desc.name = id.to_string();
                }
                self.stats.add(MaterialDesc::estimated_size());
                id
            }
        }
    }

    fn find_duplicate(&self, desc: &MaterialDesc) -> Option<MaterialId> {
        self.storage.iter().find_map(
            |(id, asset)| {
                if asset.desc == *desc { Some(id) } else { None }
            },
        )
    }

    pub fn get_desc(&self, id: MaterialId) -> Option<&MaterialDesc> {
        self.storage.get(id).map(|m| &m.desc)
    }

    pub fn contains_key(&self, id: MaterialId) -> bool {
        self.storage.contains_key(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (MaterialId, &MaterialDesc)> {
        self.storage.iter().map(|(id, asset)| (id, &asset.desc))
    }

    pub fn remove(&mut self, id: MaterialId, texture_asset: &mut TextureAssets) {
        if let Some(asset) = self.storage.get(id) {
            let count = asset.ref_count.get();

            if count > 1 {
                asset.ref_count.set(count - 1);
                self.stats.remove_sahred();
            } else {
                let removed = self.storage.remove(id).unwrap();
                let desc = removed.desc;
                // remove textures from slots
                // TODO: Remove from here
                for slot in MaterialTextureSlot::ALL {
                    if let Some(id) = desc.get_texture_slot(slot) {
                        texture_asset.remove(id);
                        debug!("Remove texture slot {:?}", slot);
                    }
                }
                debug!("Remove material id {:?}", id);
                self.stats.remove(MaterialDesc::estimated_size());
            }
        }
    }

    pub fn update(&mut self, id: MaterialId, desc: &MaterialDesc) {
        if let Some(asset) = &mut self.storage.get_mut(id) {
            asset.desc = desc.clone();
        } else {
            warn!("material id {} not found", id);
        }
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
    use super::*;

    #[test]
    fn should_create_material() {
        let mut materials = MaterialAssets::default();

        let desc = MaterialDesc::default();

        let id = materials.get_or_create(desc);

        assert!(materials.contains_key(id));
    }

    #[test]
    fn should_remove_material() {
        let mut materials = MaterialAssets::default();
        let mut texture_asset = TextureAssets::new();

        let desc = MaterialDesc::default();

        let _ = materials.get_or_create(desc.clone());
        let id = materials.get_or_create(desc);

        materials.remove(id, &mut texture_asset);
        assert!(materials.contains_key(id));
        assert!(materials.get_desc(id).is_some());

        materials.remove(id, &mut texture_asset);
        assert_eq!(materials.contains_key(id), false);
        assert!(materials.get_desc(id).is_none());
    }

    #[test]
    fn should_remove_textures_from_slot() {
        let mut materials = MaterialAssets::default();
        let mut texture_asset = TextureAssets::new();
        let path = Some(PathBuf::from("albedo.png"));

        let mut desc = MaterialDesc::default();
        desc.set_texture(&mut texture_asset, MaterialTextureSlot::BaseColor, path);

        let id = materials.get_or_create(desc);
        let mat_desc = materials.get_desc(id).unwrap();

        let tex_id = mat_desc
            .get_texture_slot(MaterialTextureSlot::BaseColor)
            .unwrap();
        assert!(texture_asset.contains_key(tex_id));

        materials.remove(id, &mut texture_asset);
        assert_eq!(texture_asset.contains_key(tex_id), false);
    }

    #[test]
    fn should_have_stats() {
        fn assert_impl<T: HasStats>() {}
        assert_impl::<MaterialAssets>();
    }

    #[test]
    fn should_increment_stats_on_add() {
        let mut materials = MaterialAssets::default();
        let initial_stats = materials.get_stats();

        let desc = MaterialDesc::default();

        let _ = materials.get_or_create(desc);

        let new_stats = materials.get_stats();

        assert!(new_stats.count > initial_stats.count);
        assert!(new_stats.estimated_bytes > initial_stats.estimated_bytes);
    }

    #[test]
    fn should_decrements_stats_on_remove() {
        let mut materials = MaterialAssets::default();
        let mut textures = TextureAssets::new();

        let initial_stats = materials.get_stats();

        let desc = MaterialDesc::default();

        let id = materials.get_or_create(desc);

        materials.remove(id, &mut textures);
        let new_stats = materials.get_stats();

        assert_eq!(new_stats.count, initial_stats.count);
        assert_eq!(new_stats.estimated_bytes, initial_stats.estimated_bytes);
    }

    #[test]
    fn should_not_remove_shared_from_asset() {
        let mut materials = MaterialAssets::default();
        let mut textures = TextureAssets::new();

        let initial_stats = materials.get_stats();

        let desc = MaterialDesc::default();

        let _ = materials.get_or_create(desc.clone());
        let id = materials.get_or_create(desc);

        materials.remove(id, &mut textures);

        // now will remove ..
        materials.remove(id, &mut textures);
        let new_stats = materials.get_stats();

        assert_eq!(new_stats.count, initial_stats.count);
        assert_eq!(new_stats.estimated_bytes, initial_stats.estimated_bytes);
    }
}
