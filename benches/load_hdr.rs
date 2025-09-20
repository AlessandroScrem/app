use criterion::{Criterion, criterion_group, criterion_main};
use rayon::prelude::*;
use std::fs;

// use half::f16;
// fn load_hdr(path: &str) -> (Vec<f32>, u32, u32) {
// use stb_image::image::{LoadResult, load_from_memory_with_depth};
//     let buffer = fs::read(path).expect("Failed to read HDR file");
//     let img = match load_from_memory_with_depth(&buffer, 4, false) {
//         LoadResult::ImageF32(img) => img,
//         LoadResult::Error(e) => panic!("Failed to load HDR: {}", e),
//         _ => panic!("Unexpected image format"),
//     };
//     (img.data, img.width as u32, img.height as u32)
// }

/*
// Versione 1: flat_map_iter + concat
fn convert_flat_map(raw_f32: &[f32]) -> Vec<u8> {
    raw_f32
        .par_chunks(4)
        .flat_map_iter(|px| {
            let r = f16::from_f32(px[0].clamp(0.0, f16::MAX.to_f32())).to_le_bytes();
            let g = f16::from_f32(px[1].clamp(0.0, f16::MAX.to_f32())).to_le_bytes();
            let b = f16::from_f32(px[2].clamp(0.0, f16::MAX.to_f32())).to_le_bytes();
            let a = f16::from_f32(1.0).to_le_bytes();
            [r, g, b, a].concat()
        })
        .collect()
}

// Versione 2: par_chunks_mut + for interno
fn convert_par_chunks_for(raw_f32: &[f32], width: u32, height: u32) -> Vec<u8> {
    let num_pixels = (width * height) as usize;
    let mut raw_u8 = vec![0u8; num_pixels * 4 * 2];

    raw_u8.par_chunks_mut(8).enumerate().for_each(|(i, chunk)| {
        let base = i * 4;
        for c in 0..4 {
            let val = if c < 3 { raw_f32[base + c] } else { 1.0 };
            let clamped = val.clamp(0.0, f16::MAX.to_f32());
            let bytes = f16::from_f32(clamped).to_le_bytes();
            chunk[c * 2..c * 2 + 2].copy_from_slice(&bytes);
        }
    });

    raw_u8
}

// Versione 3: par_chunks_mut + zip (più performante)
fn convert_par_chunks_zip(raw_f32: &[f32], width: u32, height: u32) -> Vec<u8> {
    let num_pixels = (width * height) as usize;
    let mut raw_u8 = vec![0u8; num_pixels * 4 * 2];

    raw_u8
        .par_chunks_mut(8)
        .zip(raw_f32.par_chunks(4))
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

    raw_u8
}

// Benchmark
fn _bench_conversions(c: &mut Criterion) {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/core/clarens_night_02_2k.hdr"
    );
    let (raw_f32, width, height) = load_hdr(path);

    c.bench_function("flat_map_iter + concat", |b| {
        b.iter(|| convert_flat_map(&raw_f32))
    });

    c.bench_function("par_chunks_mut + for", |b| {
        b.iter(|| convert_par_chunks_for(&raw_f32, width, height))
    });

    c.bench_function("par_chunks_mut + zip", |b| {
        b.iter(|| convert_par_chunks_zip(&raw_f32, width, height))
    });
} */

fn load_hdr_to_buffer_u8(path: &str) -> Vec<u8> {
    fs::read(path).expect("Failed to read HDR file")
}

fn decode_image_rs_serial(buffer: &[u8]) -> (Vec<u8>, u32, u32) {
    let image = image::load_from_memory(buffer).unwrap().to_rgba32f();
    let (width, height) = image.dimensions();
    let raw_f32: Vec<f32> = image.into_raw();
    // conversione diretta in Vec<u8> per Rgba16Float
    // fit 32bit value to range [0.0 65504.0]
    let raw_u8: Vec<u8> = raw_f32
        .iter()
        .flat_map(|f| {
            let clamped = f.clamp(0.0, half::f16::MAX.to_f32());
            half::f16::from_f32(clamped).to_le_bytes()
        })
        .collect();
    (raw_u8, width, height)
}

fn decode_image_rs_parallel(buffer: &[u8]) -> (Vec<u8>, u32, u32) {
    use rayon::iter::ParallelIterator;
    let image = image::load_from_memory(buffer).unwrap().to_rgba32f();
    let (width, height) = image.dimensions();
    let raw_f32: Vec<f32> = image.into_raw();

    // conversione parallela in Vec<u8> per Rgba16Float
    // fit 32bit value to range [0.0 65504.0]

    let raw_u8: Vec<u8> = raw_f32
        .par_iter() // Rayon parallel iterator
        .flat_map_iter(|f| {
            let clamped = f.clamp(0.0, half::f16::MAX.to_f32());
            half::f16::from_f32(clamped).to_le_bytes()
        })
        .collect();

    (raw_u8, width, height)
}

fn decode_stb_image_parallel(buffer: &[u8]) -> (Vec<u8>, u32, u32) {
    use half::f16;
    use rayon::prelude::*;
    use stb_image::image::LoadResult;
    use stb_image::image::load_from_memory_with_depth;
    // Caricamento HDR con stb_image

    let img = match load_from_memory_with_depth(buffer, 4, false) {
        LoadResult::ImageF32(img) => img,
        LoadResult::Error(e) => panic!("Failed: {}", e),
        _ => panic!("Unexpected format"),
    };

    let width = img.width as u32;
    let height = img.height as u32;
    let raw_f32 = img.data; // già in f32, RGB/RGBA

    // Conversione parallela in Vec<u8> per Rgba16Float
    let raw_u8: Vec<u8> = raw_f32
        .par_iter()
        .flat_map_iter(|f| {
            let clamped = f.clamp(0.0, f16::MAX.to_f32());
            f16::from_f32(clamped).to_le_bytes()
        })
        .collect();

    (raw_u8, width, height)
}

fn decode_stb_image_parallel2(buffer: &[u8]) -> (Vec<u8>, u32, u32) {
    use half::f16;
    use rayon::prelude::*;
    use stb_image::image::LoadResult;
    use stb_image::image::load_from_memory_with_depth;
    // Caricamento HDR con stb_image

    let img = match load_from_memory_with_depth(buffer, 4, false) {
        LoadResult::ImageF32(img) => img,
        LoadResult::Error(e) => panic!("Failed: {}", e),
        _ => panic!("Unexpected format"),
    };

    let width = img.width as u32;
    let height = img.height as u32;
    let num_pixels = (width * height) as usize;

    // Prealloca il buffer finale: 4 canali * 2 byte per pixel
    let mut raw_u8 = vec![0u8; num_pixels * 4 * 2];

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

    (raw_u8, width, height)
}

// Benchmark
fn bench_conversions(c: &mut Criterion) {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/core/clarens_night_02_2k.hdr"
    );
    let raw_u8 = load_hdr_to_buffer_u8(path);

    c.bench_function("image_rs ser", |b| {
        b.iter(|| decode_image_rs_serial(&raw_u8))
    });
    c.bench_function("image_rs par", |b| {
        b.iter(|| decode_image_rs_parallel(&raw_u8))
    });
    c.bench_function("stb_image flat_map_iter + collect", |b| {
        b.iter(|| decode_stb_image_parallel(&raw_u8))
    });
    c.bench_function("stb_image par_chunks_mut + zip", |b| {
        b.iter(|| decode_stb_image_parallel2(&raw_u8))
    });
}

fn load_image_rs(buffer: &[u8]) -> (Vec<f32>, u32, u32) {
    let image = image::load_from_memory(buffer).unwrap().to_rgba32f();
    let (width, height) = image.dimensions();
    let raw_f32: Vec<f32> = image.into_raw();

    (raw_f32, width, height)
}

fn load_stb_image(buffer: &[u8]) -> (Vec<f32>, u32, u32) {
    use stb_image::image::LoadResult;
    use stb_image::image::load_from_memory_with_depth;
    // Caricamento HDR con stb_image

    let img = match load_from_memory_with_depth(buffer, 4, false) {
        LoadResult::ImageF32(img) => img,
        LoadResult::Error(e) => panic!("Failed: {}", e),
        _ => panic!("Unexpected format"),
    };

    let width = img.width as u32;
    let height = img.height as u32;
    let raw_f32 = img.data; // già in f32, RGB/RGBA

    (raw_f32, width, height)
}



// Benchmark
fn bench_loaders(c: &mut Criterion) {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/core/clarens_night_02_2k.hdr"
    );
    let raw_u8 = load_hdr_to_buffer_u8(path);

    c.bench_function("load mage_rs ", |b| {
        b.iter(|| load_image_rs(&raw_u8))
    });
    c.bench_function("load stb_image", |b| {
        b.iter(|| load_stb_image(&raw_u8))
    });

}

criterion_group!(benches, bench_conversions, bench_loaders);
criterion_main!(benches);
