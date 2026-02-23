use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;

use super::TextureId;
use super::file;
use crate::assets::TextureState;
use crate::assets::{ColorSpace, TextureAsset, TextureDesc};
use crate::prelude::*;

use super::image_decoder::{
    decode_image_rgbaf32, decode_stb_image_rgaba8, decode_stb_image_rgbaf16,
};

#[derive(Clone)]
pub(crate) struct CpuTexture {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) pixels: Vec<u8>,
    pub(crate) format: ColorSpace,
}

/// --- White texture 1x1 RGBA8 
impl CpuTexture {
    pub(crate) fn white() -> Self {
        Self {
            width: 1,
            height: 1,
            format: ColorSpace::Rgba8,
            pixels: vec![255, 255, 255, 255],
        }
    }
}

pub(crate) enum UploadPayload {
    Ready(CpuTexture),
    Fallback,
}

pub(crate) trait TextureUploadSource {
    fn drain_dirty_textures(&mut self) -> Vec<(TextureId, UploadPayload)>;

    fn get_texture_asset(&self, id: TextureId) -> Option<&TextureAsset>;
}

#[derive(Debug)]
pub(crate) enum TextureError {
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

pub(crate) fn load_cpu_textures_par<'a>(
    textures: impl Iterator<Item = (TextureId, &'a TextureAsset)>,
) -> Vec<(TextureId, Result<UploadPayload, TextureError>)> {
    // collect texture MetaOnly
    let jobs: Vec<_> = textures
        .filter_map(|(id, tex)| {
            if tex.state != TextureState::MetaOnly {
                return None;
            }
            Some((id, tex.desc.clone()))
        })
        .collect();

    // spawn decode on thread pool Rayon
    let results: Vec<_> = jobs
        .into_par_iter() // parallelo
        .map(|(id, desc)| {
            let result = load_and_decode(desc);
            (id, result)
        })
        .collect();

    results
}

fn load_and_decode(desc: Option<TextureDesc>) -> Result<UploadPayload, TextureError> {
    let desc = match desc {
        Some(d) => d,
        None => {
            return Ok(UploadPayload::Fallback);
        }
    };

    let (path, color_space) = match desc {
        TextureDesc::File { key, .. } => match key {
            assets::TextureKey::File {
                path, color_space, ..
            } => (path, color_space),
            assets::TextureKey::White => return Ok(UploadPayload::Fallback),
        },
    };

    let buffer = file::read_bytes(path)?;

    let (pixels, width, height) = match color_space {
        ColorSpace::Rgba8 | ColorSpace::Srgba8 => decode_stb_image_rgaba8(&buffer)?,

        ColorSpace::Rgbaf16 => decode_stb_image_rgbaf16(&buffer)?,

        ColorSpace::Rgbaf32 => decode_image_rgbaf32(&buffer)?,
    };

    Ok(UploadPayload::Ready(CpuTexture {
        format: color_space.clone(),
        width,
        height,
        pixels,
    }))
}
