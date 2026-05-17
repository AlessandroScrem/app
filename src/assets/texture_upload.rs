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
pub struct TextureData {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub format: ColorSpace,
}

impl TextureData {
    pub fn estimated_size(&self) -> usize {
        self.pixels.len()
    }
}

pub enum UploadPayload {
    Ready(TextureData),
    Fallback,
}

pub trait TextureUploadSource {
    fn drain_dirty_textures(&mut self) -> Vec<(TextureId, UploadPayload)>;

    fn get_texture_asset(&self, id: TextureId) -> Option<&TextureAsset>;
}

pub fn load_cpu_textures_par<'a>(
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
        TextureDesc::File { path, usage, .. } => (path, usage.color_space()),

        TextureDesc::White => {
            return Ok(UploadPayload::Fallback);
        }
    };

    trace!("read texture {:?}", path.as_path());

    let buffer = file::read_bytes(path)?;

    let (pixels, width, height) = match color_space {
        ColorSpace::Rgba8 | ColorSpace::Srgba8 => decode_stb_image_rgaba8(&buffer)?,
        ColorSpace::Rgbaf16 => decode_stb_image_rgbaf16(&buffer)?,
        ColorSpace::Rgbaf32 => decode_image_rgbaf32(&buffer)?,
        ColorSpace::Rg32ui => unimplemented!(),
        ColorSpace::Depth32f => unimplemented!(),
    };

    Ok(UploadPayload::Ready(TextureData {
        format: color_space,
        width,
        height,
        pixels,
    }))
}
