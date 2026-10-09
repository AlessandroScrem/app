use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;

use super::TextureId;
use super::file;
use crate::assets::texture_asset::{ColorSpace, TextureDesc};
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

pub fn load_cpu_textures_par<'a>(
    jobs: Vec<(TextureId, TextureDesc)>,
) -> Vec<(TextureId, TextureData)> {
    // spawn decode on thread pool Rayon
    let results: Vec<_> = jobs
        .into_par_iter() // parallelo
        .filter_map(|(id, desc)| load_and_decode(desc).map(|data| (id, data)))
        .collect();

    results
}

pub fn load_and_decode(desc: TextureDesc) -> Option<TextureData> {
    let (path, color_space) = match desc {
        TextureDesc::File { path, usage, .. } => (path, usage.color_space()),
    };

    trace!("read texture {:?}", path.as_path());

    let buffer = file::read_bytes(path).ok()?;

    let (pixels, width, height) = match color_space {
        ColorSpace::Rgba8 | ColorSpace::Srgba8 => decode_stb_image_rgaba8(&buffer).ok()?,
        ColorSpace::Rgbaf16 => decode_stb_image_rgbaf16(&buffer).ok()?,
        ColorSpace::Rgbaf32 => decode_image_rgbaf32(&buffer).ok()?,
        ColorSpace::Rg32ui => unimplemented!(),
        ColorSpace::Depth32f => unimplemented!(),
    };

    Some(TextureData {
        format: color_space,
        width,
        height,
        pixels,
    })
}
