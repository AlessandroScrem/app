use super::*;
use crate::math::*;
use std::ops::{Index, IndexMut};
use crate::uniform::Mat3Std140;

pub const IOR: f32 = 1.5;
pub const MATERIAL_TEXTURE_COUNT: usize = 7;
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


#[derive(Default, Eq, PartialEq, Debug, Clone)]
pub struct TextureSlot {
    texture: Option<TextureId>,
    coord: u32,
    transform: Option<TextureTransform>,
    enabled: bool,
}

#[derive(Debug, Default, Clone)]
pub struct TextureSet {
    slot: [TextureSlot; MATERIAL_TEXTURE_COUNT],
}

///
/// TEXTURE TRANSFORM
///
#[derive(Clone, Copy, Debug)]
pub struct TextureTransform {
    pub offset: [f32; 2],
    pub rotation: f32,
    pub scale: [f32; 2],
}

///
/// ALPHA MODE
///
#[derive(Default, Copy, Clone, Debug)]
pub enum AlphaMode {
    #[default]
    Opaque,
    Mask {
        alpha_cutoff: f32,
    },
    Blend,
}

///
/// VOLUME
///
#[derive(Debug, Default, Clone, Copy)]
pub struct Volume {
    pub thickness_factor: f32,
    pub attenuation_distance: f32,
    pub attenuation_color: [f32; 3],
}

///
/// TRANSMISSION
///
#[derive(Debug, Default, Clone, Copy)]
pub struct Transmission {
    pub factor: f32,
}

///
/// SHEEN
///
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct Sheen {
    pub color_factor: [f32; 3],
    pub roughness_factor: f32,
}


#[derive(Debug, Clone)]
pub struct MaterialDesc {
    pub name: String,
    #[allow(unused)]
    shader: ShaderId,

    pub texture_set: TextureSet,

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
    pub sheen: Option<Sheen>,
}

impl Default for MaterialDesc {
    fn default() -> Self {
        MaterialDesc {
            name: String::new(),
            shader: ShaderId::default(),
            texture_set: TextureSet::default(),

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

impl PartialEq for MaterialDesc {
    fn eq(&self, other: &Self) -> bool {
        self.texture_set.slot == other.texture_set.slot
            && self.alpha_mode == other.alpha_mode
            && self.transmission == other.transmission
            && self.volume == other.volume
            && self.base_color_factor == other.base_color_factor
            && self.emissive_factor == other.emissive_factor
            && self.roughness_factor.to_bits() == other.roughness_factor.to_bits()
            && self.metallic_factor.to_bits() == other.metallic_factor.to_bits()
            && self.normal_scale.to_bits() == other.normal_scale.to_bits()
            && self.occlusion_strength.to_bits() == other.occlusion_strength.to_bits()
            && self.ior.to_bits() == other.ior.to_bits()
            && self.sheen == other.sheen
    }
}

impl MaterialDesc {
    pub fn texture(&self, slot: MaterialTextureSlot) -> Option<TextureId> {
        self.texture_set[slot].texture
    }

    pub fn uvtransform(&self, slot: MaterialTextureSlot) -> Option<TextureTransform> {
        self.texture_set[slot].transform
    }

    pub fn uvtransform_mut(&mut self, slot: MaterialTextureSlot) -> Option<&mut TextureTransform> {
        self.texture_set[slot].transform.as_mut()
    }

    pub fn estimated_size() -> usize {
        size_of::<MaterialDesc>()
    }

    pub fn slot_get(&self, slot: MaterialTextureSlot) -> bool {
        self.texture_set[slot].enabled
    }
    pub fn slot_set(&mut self, slot: MaterialTextureSlot, enabled: bool) {
        self.texture_set[slot].enabled = enabled;
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

    pub fn is_sheen(&self) -> bool {
        self.sheen.is_some()
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
            self.texture_set[slot].texture = Some(id);
            self.texture_set[slot].transform = transform;
            self.texture_set[slot].coord = coord;
            self.texture_set[slot].enabled = true;
        }
    }
}


impl Index<MaterialTextureSlot> for TextureSet {
    type Output = TextureSlot;

    fn index(&self, slot: MaterialTextureSlot) -> &Self::Output {
        &self.slot[slot as usize]
    }
}

impl IndexMut<MaterialTextureSlot> for TextureSet {
    fn index_mut(&mut self, slot: MaterialTextureSlot) -> &mut Self::Output {
        &mut self.slot[slot as usize]
    }
}

impl PartialEq for TextureSet {
    fn eq(&self, other: &Self) -> bool {
        self.slot.iter().zip(other.slot.iter()).all(|(a, b)| {
            a.texture == b.texture && a.coord == b.coord && a.transform == b.transform
        })
    }
}
impl Eq for TextureSet {}

impl TextureSet {
    pub fn texture_flags(&self) -> u32 {
        let mut flags = 0;

        for s in MaterialTextureSlot::ALL {
            if self[s].texture.is_some() && self[s].enabled {
                flags |= s.bit();
            }
        }

        flags
    }

    pub fn coord_flags(&self) -> u32 {
        let mut flags = 0;

        for s in MaterialTextureSlot::ALL {
            if self[s].coord > 0 {
                flags |= s.bit();
            }
        }

        flags
    }
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


impl Sheen {
    pub fn to_uniform(opt: Option<Self>) -> ([f32; 3], f32) {
        opt.map_or(([0.0; 3], 0.0), |s| (s.color_factor, s.roughness_factor))
    }
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
