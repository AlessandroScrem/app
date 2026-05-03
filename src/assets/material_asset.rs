use super::*;
use crate::uniform::Mat3Std140;
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

use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, Debug)]
pub struct TextureTransform {
    pub offset: [f32; 2],
    pub rotation: f32,
    pub scale: [f32; 2],
}
impl Default for TextureTransform {
    fn default() -> Self {
        Self {
            offset: [0.0, 0.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
        }
    }
}
impl TextureTransform {
    pub fn to_mat3_std140(&self) -> Mat3Std140 {
        let t = Mat3::from_translation(self.offset.into());
        let r = Mat3::from_angle_z(Rad(self.rotation));
        let s = Mat3::from_nonuniform_scale(self.scale[0], self.scale[1]);
        let m = t * r * s;

        Mat3Std140::mat3_to_std140(m)
    }
}

impl Hash for TextureTransform {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for v in self.offset {
            v.to_bits().hash(state);
        }
        self.rotation.to_bits().hash(state);
        for v in self.scale {
            v.to_bits().hash(state);
        }
    }
}

impl PartialEq for TextureTransform {
    fn eq(&self, other: &Self) -> bool {
        self.offset[0].to_bits() == other.offset[0].to_bits()
            && self.offset[1].to_bits() == other.offset[1].to_bits()
            && self.rotation.to_bits() == other.rotation.to_bits()
            && self.scale[0].to_bits() == other.scale[0].to_bits()
            && self.scale[1].to_bits() == other.scale[1].to_bits()
    }
}

impl Eq for TextureTransform {}

#[derive(Debug, Default, Hash, Eq, PartialEq, Clone)]
pub struct TestureSet {
    textures: [Option<TextureId>; MATERIAL_TEXTURE_COUNT],
    transforms: [Option<TextureTransform>; MATERIAL_TEXTURE_COUNT],
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

pub const IOR: f32 = 1.5;
pub const MATERIAL_TEXTURE_COUNT: usize = 7;
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum MaterialTextureSlot {
    BaseColor = 0,
    Normal = 1,
    MetallicRoughness = 2,
    Emissive = 3,
    Occlusion = 4,
    Transmission = 5,
    Volume = 6,
}

impl MaterialTextureSlot {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BaseColor => "Base Color",
            Self::Normal => "Normal",
            Self::MetallicRoughness => "Metallic Roughness",
            Self::Emissive => "Emissive",
            Self::Occlusion => "Occlusion",
            Self::Transmission => "Transmission",
            Self::Volume => "Volume",
        }
    }
}

impl MaterialTextureSlot {
    #[inline]
    pub const fn bit(self) -> u32 {
        1 << (self as u32)
    }
}

impl MaterialTextureSlot {
    pub fn color_space(self) -> ColorSpace {
        match self {
            Self::BaseColor | Self::Transmission | Self::Emissive => ColorSpace::Srgba8,
            Self::Normal | Self::MetallicRoughness | Self::Occlusion | Self::Volume => {
                ColorSpace::Rgba8
            }
        }
    }
}

impl MaterialTextureSlot {
    pub const ALL: [MaterialTextureSlot; MATERIAL_TEXTURE_COUNT] = [
        Self::BaseColor,
        Self::Normal,
        Self::MetallicRoughness,
        Self::Emissive,
        Self::Occlusion,
        Self::Transmission,
        Self::Volume,
    ];
}

#[repr(u8)]
#[derive(Default, Copy, Clone, Debug)]
pub enum AlphaMode {
    #[default]
    Opaque,
    Mask {
        alpha_cutoff: f32,
    },
    Blend,
}
impl AlphaMode {
    pub fn to_uniform(mode: Self) -> (u32, f32) {
        match mode {
            AlphaMode::Opaque => (0, 0.0),
            AlphaMode::Mask { alpha_cutoff } => (1, alpha_cutoff),
            AlphaMode::Blend => (2, 0.0),
        }
    }

    pub fn mask_default() -> Self {
        AlphaMode::Mask { alpha_cutoff: 0.5 }
    }
}

impl Eq for AlphaMode {}
impl PartialEq for AlphaMode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AlphaMode::Opaque, AlphaMode::Opaque) => true,
            (AlphaMode::Blend, AlphaMode::Blend) => true,
            (AlphaMode::Mask { alpha_cutoff: a }, AlphaMode::Mask { alpha_cutoff: b }) => {
                a.to_bits() == b.to_bits()
            }
            _ => false,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Volume {
    pub thickness_factor: f32,
    pub attenuation_distance: f32,
    pub attenuation_color: [f32; 3],
}
impl PartialEq for Volume {
    fn eq(&self, other: &Self) -> bool {
        self.thickness_factor.to_bits() == other.thickness_factor.to_bits()
            && self.attenuation_distance.to_bits() == other.attenuation_distance.to_bits()
            && self.attenuation_color == other.attenuation_color
    }
}

impl Volume {
    pub fn to_uniform(opt: Option<Self>) -> (f32, f32, [f32; 3]) {
        opt.map_or((f32::INFINITY, 0.0, [1.0; 3]), |t| {
            (
                t.attenuation_distance,
                t.thickness_factor,
                t.attenuation_color,
            )
        })
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Transmission {
    pub factor: f32,
}

impl PartialEq for Transmission {
    fn eq(&self, other: &Self) -> bool {
        self.factor.to_bits() == other.factor.to_bits()
    }
}

impl Transmission {
    pub fn to_uniform(opt: Option<Self>) -> f32 {
        opt.map_or(0.0, |t| t.factor)
    }
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct MaterialSheen {
    pub color_factor: [f32; 3],
    pub roughness_factor: f32,
}


#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TextureFlags {
    flags: u32,
}

impl std::fmt::Debug for TextureFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let slots = [
            MaterialTextureSlot::BaseColor,
            MaterialTextureSlot::Normal,
            MaterialTextureSlot::MetallicRoughness,
            MaterialTextureSlot::Emissive,
            MaterialTextureSlot::Occlusion,
            MaterialTextureSlot::Transmission,
            MaterialTextureSlot::Volume,
        ];

        for (i, slot) in slots.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }

            let bit = 1 << (*slot as u32);

            if self.flags & bit != 0 {
                write!(f, "{:?}", slot)?; // stampa nome enum
            } else {
                write!(f, "None")?;
            }
        }

        Ok(())
    }
}

impl TextureFlags {
    pub fn new() -> Self {
        Self { flags: 0 }
    }

    #[inline]
    pub fn get(&self, slot: MaterialTextureSlot) -> bool {
        (self.flags & slot.bit()) != 0
    }

    #[inline]
    pub fn set(&mut self, slot: MaterialTextureSlot, enabled: bool) {
        if enabled {
            self.flags |= slot.bit();
        } else {
            self.flags &= !slot.bit();
        }
    }

    #[allow(unused)]
    #[inline]
    pub fn clear(&mut self) {
        self.flags = 0;
    }

    #[inline]
    pub fn raw(&self) -> u32 {
        self.flags
    }

    #[allow(unused)]
    #[inline]
    pub fn from_raw(flags: u32) -> Self {
        Self { flags }
    }
}

#[derive(Debug, Clone)]
pub struct MaterialDesc {
    name: String,
    #[allow(unused)]
    shader: ShaderId,

    pub texture_set: TestureSet,
    texture_flags: TextureFlags,
    coord_flags: TextureFlags,

    pub alpha_mode: AlphaMode,
    pub base_color_factor: Vec4,
    pub emissive_factor: Vec4,
    pub roughness_factor: f32,
    pub metallic_factor: f32,
    pub normal_scale: f32,
    pub occlusion_strength: f32,
    pub transmission: Option<Transmission>,
    pub volume: Option<Volume>,
    pub ior: f32,
    pub sheen: Option<MaterialSheen>,
}

// PartialEq ignore: name , shader
impl PartialEq for MaterialDesc {
    fn eq(&self, other: &Self) -> bool {
        self.texture_set.textures == other.texture_set.textures
            && self.texture_flags == other.texture_flags
            && self.alpha_mode == other.alpha_mode
            && self.transmission == other.transmission
            && self.volume == other.volume
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
            && self.ior.to_bits() == other.ior.to_bits()
            && self.sheen == other.sheen
    }
}

impl Default for MaterialDesc {
    fn default() -> Self {
        MaterialDesc {
            name: String::new(),
            shader: ShaderId::default(),
            texture_set: TestureSet::default(),
            texture_flags: TextureFlags::new(),
            coord_flags: TextureFlags::new(),

            alpha_mode: AlphaMode::default(),
            base_color_factor: Vec4::from_value(one()),
            metallic_factor: one(),
            roughness_factor: one(),
            emissive_factor: Vec4::from_value(zero()),
            normal_scale: one(),
            occlusion_strength: one(),
            transmission: None,
            volume: None,
            ior: IOR,
            sheen: None,
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

    pub fn get_uvtransform_slot(&self, slot: MaterialTextureSlot) -> Option<TextureTransform> {
        self.texture_set
            .transforms
            .get(slot as usize)
            .copied()
            .flatten()
    }

    pub fn get_uvtransform_slot_mut(
        &mut self,
        slot: MaterialTextureSlot,
    ) -> Option<&mut TextureTransform> {
        self.texture_set
            .transforms
            .get_mut(slot as usize)
            .and_then(|opt| opt.as_mut())
    }

    fn estimated_size() -> usize {
        size_of::<MaterialDesc>()
    }

    pub fn slot_get(&self, slot: MaterialTextureSlot) -> bool {
        self.texture_flags.get(slot)
    }
    pub fn slot_set(&mut self, slot: MaterialTextureSlot, enabled: bool) {
        self.texture_flags.set(slot, enabled);
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn set_name(&mut self, name: &str) {
        self.name = name.into();
    }

    pub fn is_transmissive(&self) -> bool {
        self.transmission.map(|t| t.factor > 0.0).unwrap_or(false)
    }

    pub fn is_transparent(&self) -> bool {
        match self.alpha_mode {
            AlphaMode::Blend => true,
            _ => false,
        }
    }

    pub fn is_volume(&self) -> bool {
        self.volume
            .map(|f| f.attenuation_distance > 0.0)
            .unwrap_or(false)
    }

    pub fn set_texture(
        &mut self,
        texture_asset: &mut TextureAssets,
        slot: MaterialTextureSlot,
        path: Option<PathBuf>,
        coord: u32,
        transform: Option<TextureTransform>,
    ) {
        if let Some(path) = path {
            let key = TextureKey::File {
                color_space: slot.color_space().into(),
                path: path.into(),
                usage: slot.into(),
            };
            let desc = super::TextureDesc::File {
                key,
                sampler: super::SamplerDesc::LinearRepeat,
                mipmaps: false,
            };
            let id = texture_asset.get_or_create(desc);
            self.texture_set.textures[slot as usize] = Some(id);
            self.texture_set.transforms[slot as usize] = transform;
            self.coord_flags.set(slot, coord > 0);
            self.texture_flags.set(slot, true);
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
            debug!(
                "Update material id {:?} with desc {:?}",
                id,
                desc.get_uvtransform_slot(MaterialTextureSlot::BaseColor)
            );
            asset.desc = desc.clone();
        } else {
            warn!("material id {} not found", id);
        }
    }
}

fn gen_transform_array(desc: &MaterialDesc) -> [Mat3Std140; MATERIAL_TEXTURE_COUNT] {
    std::array::from_fn(|i| {
        let slot = MaterialTextureSlot::ALL[i];

        desc.get_uvtransform_slot(slot)
            .unwrap_or_default()
            .to_mat3_std140()
    })
}

impl From<&MaterialDesc> for MaterialUniform {
    fn from(value: &MaterialDesc) -> Self {
        let (alpha_mode, alpha_cutoff) = AlphaMode::to_uniform(value.alpha_mode);
        let is_trasmissive = value.is_transmissive().into();
        let transmission_factor = Transmission::to_uniform(value.transmission);

        let is_volume = value.is_volume().into();
        let (attenuation_distance, thickness_factor, attenuation_color) =
            Volume::to_uniform(value.volume);
        let texture_transforms = gen_transform_array(value);


        Self {
            color_factor: value.base_color_factor.into(),
            emissive_factor: value.emissive_factor.into(),
            metallic_factor: value.metallic_factor,
            roughness_factor: value.roughness_factor,
            normal_scale: value.normal_scale,
            occlusion_strength: value.occlusion_strength,
            texture_flags: value.texture_flags.raw(),
            alpha_mode,
            alpha_cutoff,
            transmission_factor,
            is_trasmissive,
            is_volume,
            attenuation_distance,
            thickness_factor,
            attenuation_color,
            ior: value.ior,
            texture_transforms,
            coord_flags: value.coord_flags.raw(),
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
        desc.set_texture(
            &mut texture_asset,
            MaterialTextureSlot::BaseColor,
            path,
            0,
            None,
        );

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
