use crate::assets::global_asset_manager::asset_storage::Asset;
use std::path::PathBuf;

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum TextureUsage {
    Albedo,
    Normal,
    MetallicRoughness,
    Emissive,
    Occlusion,
    Transmission,
    Volume,
    HDR16,
    #[allow(unused)]
    HDR32,
}

impl TextureUsage {
    pub fn color_space(self) -> ColorSpace {
        match self {
            Self::Albedo | Self::Emissive | Self::Transmission => ColorSpace::Srgba8,
            Self::Normal | Self::Occlusion | Self::MetallicRoughness | Self::Volume => {
                ColorSpace::Rgba8
            }
            Self::HDR16 => ColorSpace::Rgbaf16,
            Self::HDR32 => ColorSpace::Rgbaf32,
        }
    }
}

#[derive(Default, Hash, Eq, PartialEq, Clone, Debug)]
pub enum SamplerDesc {
    #[default]
    LinearRepeat,
    LinearMipmap,
    Nearest,
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum TextureDesc {
    File {
        path: PathBuf,
        usage: TextureUsage,
        sampler: SamplerDesc,
        mipmaps: bool,
    },
    White,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum ColorSpace {
    Rgbaf32,
    Rgbaf16,
    Srgba8,
    Rgba8,
    Rg32ui,
    Depth32f,
}

impl ColorSpace {
    // pixel size (Bytes)
    pub fn pixel_size(&self) -> u32 {
        match self {
            Self::Rgbaf32 => 16,
            Self::Rgbaf16 | Self::Rg32ui => 8,
            Self::Depth32f | Self::Srgba8 | Self::Rgba8 => 4,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct TextureInfo {
    pub width: u32,
    pub height: u32,
    pub format: ColorSpace,
    pub byte_size: usize,
}

#[derive(Clone, PartialEq, Debug)]
pub enum TextureState {
    MetaOnly,              // solo path noto
    CpuReady(TextureInfo), // cpu texture caricata
    Fallback,              // errore → fallback
}

impl TextureState {
    pub fn estimated_size(&self) -> usize {
        match self {
            TextureState::CpuReady(i) => i.byte_size,
            _ => 0,
        }
    }
}

#[derive(Clone)]
pub struct TextureAsset {
    pub state: TextureState,
    pub desc: TextureDesc,
}

impl Asset for TextureAsset {
    type Key = TextureDesc;

    fn key(&self) -> &Self::Key {
        &self.desc
    }
}

pub fn create_texture(path: std::path::PathBuf, usage: TextureUsage) -> TextureAsset {
    let desc = TextureDesc::File {
        path,
        usage,
        sampler: SamplerDesc::default(),
        mipmaps: true,
    };

    TextureAsset {
        desc: desc,
        state: TextureState::MetaOnly,
    }
}
