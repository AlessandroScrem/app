use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;

use super::TextureId;
use super::file;
use crate::assets::TextureState;
use crate::assets::{ColorSpace, TextureAsset, TextureDesc};
use crate::prelude::*;

use super::image_decoder::{decode_stb_image_par, read_stb_image};

#[derive(Clone)]
pub struct CpuTexture {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub format: ColorSpace,
}

pub enum UploadPayload {
    Ready(CpuTexture),
    Fallback,
}

pub trait TextureUploadSource {
    fn drain_dirty_textures(&mut self) -> Vec<(TextureId, UploadPayload)>;

    fn get_texture_asset(&self, id: TextureId) -> Option<&TextureAsset>;
}

#[derive(Debug)]
pub enum TextureError {
    Io(std::io::Error),
    String(String),
    Image(image::ImageError),
    FallbackWhite,
    DecodeError,
}

impl From<String> for TextureError {
    fn from(value: String) -> Self {
        TextureError::String(value)
    }
}

impl From<image::ImageError> for TextureError {
    fn from(value: image::ImageError) -> Self {
        TextureError::Image(value)
    }
}

pub fn load_cpu_textures_par<'a>(textures: impl Iterator<Item = (TextureId, &'a TextureAsset)>)->Vec<(TextureId, Result<CpuTexture, TextureError>)> {
    // 1️⃣ raccogli solo le texture MetaOnly
    let jobs: Vec<_> = 
        textures
        .filter_map(|(id, tex)| {
            if tex.state != TextureState::MetaOnly {
                return None;
            }
            Some((id, tex.desc.clone()))
        })
        .collect();

    // 2️⃣ spawn decode su thread pool Rayon
    let results: Vec<_> = jobs
        .into_par_iter() // parallelo
        .map(|(id, desc)| {
            let result = load_and_decode(&desc); // la tua funzione originale
            (id, result)
        })
        .collect();
    
    results
}

pub fn load_and_decode(desc: &TextureDesc) -> Result<CpuTexture, TextureError> {
    match desc {
        TextureDesc::File { key, .. } => match key {
            assets::TextureKey::File {
                path, color_space, ..
            } => {
                let buffer = file::read_bytes(path)?;
                match color_space {
                    ColorSpace::Rgba8 | ColorSpace::Srgba8 => {
                        let (pixels, width, height) = read_stb_image(&buffer)?;
                        Ok(CpuTexture {
                            format: color_space.clone(),
                            width,
                            height,
                            pixels,
                        })
                    }
                    ColorSpace::Rgbaf16 => {
                        let (pixels, width, height) = decode_stb_image_par(&buffer)?;
                        Ok(CpuTexture {
                            format: color_space.clone(),
                            width,
                            height,
                            pixels,
                        })
                    }
                    ColorSpace::Rgbaf32 => {
                        let image = image::load_from_memory(&buffer)?.to_rgba32f();
                        let (width, height) = image.dimensions();
                        let raw_f32: Vec<f32> = image.into_raw();
                        let pixels: Vec<u8> = bytemuck::cast_slice(&raw_f32).to_vec();
                        Ok(CpuTexture {
                            format: color_space.clone(),
                            width,
                            height,
                            pixels,
                        })
                    }
                }
            }
            assets::TextureKey::White => Err(TextureError::FallbackWhite),
        },
        TextureDesc::White => Err(TextureError::FallbackWhite),
    }
}
