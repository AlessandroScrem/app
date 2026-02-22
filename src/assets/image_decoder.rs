use super::prelude::*;
use stb_image::image::load_from_memory_with_depth;

pub fn decode_stb_image_par(buffer: &[u8]) -> Result<(Vec<u8>, u32, u32), String> {
    use half::f16;
    use rayon::prelude::*;
    use stb_image::image::LoadResult;
    // Caricamento HDR con stb_image
    const CHANNELS: usize = 4;
    const BYTE_PER_PIXEL: usize = CHANNELS * 2;

    let img = match load_from_memory_with_depth(buffer, CHANNELS, false) {
        LoadResult::ImageF32(img) => img,
        LoadResult::Error(e) => return Err(format!("Failed: {e}")),
        _ => return Err("stb_image: Unexpected format".into()),
    };

    let width = img.width as u32;
    let height = img.height as u32;
    let num_pixels = (width * height) as usize;

    let timer = std::time::Instant::now();
    
    // Prealloca il buffer finale: 4 canali * 2 byte per pixel
    let mut raw_u8 = vec![0u8; num_pixels * BYTE_PER_PIXEL];

    // Parallel map diretto: pixel source -> pixel destination
    raw_u8
        .par_chunks_mut(8) // 8 byte per pixel RGBA16
        .zip(img.data.par_chunks(4)) // 4 float per pixel RGBA, passiamo un reference
        .for_each(|(dst, src)| {
            dst[0..2].copy_from_slice(
                &f16::from_f32(src[0].clamp(0.0, f16::MAX.to_f32())).to_le_bytes(),
            );
            dst[2..4].copy_from_slice(
                &f16::from_f32(src[1].clamp(0.0, f16::MAX.to_f32())).to_le_bytes(),
            );
            dst[4..6].copy_from_slice(
                &f16::from_f32(src[2].clamp(0.0, f16::MAX.to_f32())).to_le_bytes(),
            );
            dst[6..8].copy_from_slice(
                &f16::from_f32(src[3].clamp(0.0, f16::MAX.to_f32())).to_le_bytes(),
            );
        });

    debug!(
        "Time for decoding HDR (stb_image, parallel): {:?}",
        timer.elapsed().as_millis()
    );
    Ok((raw_u8, width, height))
}

pub fn read_stb_image(buffer: &[u8]) -> Result<(Vec<u8>, u32, u32), String> {
    use stb_image::image::LoadResult;
    // Caricamento LDR con stb_image

    let timer = std::time::Instant::now();
    let img = match load_from_memory_with_depth(buffer, 4, false) {
        LoadResult::ImageU8(img) => Ok(img),
        LoadResult::Error(e) => Err(format!("Failed: {e}")),
        _ => Err("stb_image: Unexpected format".into()),
    }?;

    debug!(
        "Time for Load LDR image (stb_image, parallel): {:?}",
        timer.elapsed().as_millis()
    );

    Ok((img.data, img.width as u32, img.height as u32))
}
