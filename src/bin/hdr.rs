/* fn main() {
    println!("Hello hdr loader");

    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/core/clarens_night_02_2k.hdr"
    );

    // reading from buffer
    let buffer = std::fs::read(path).expect("unable to read file");
    let image = image::load_from_memory(&buffer).unwrap();

    // readimg from path
    // use image::{GenericImageView, ImageFormat, ImageReader};
    // let image = ImageReader::open(path).unwrap().decode().unwrap();
    // println!("Read image : {:?}", image.color());
    // println!("Size : [{} x {}]", image.width(), image.height());
    // let format = ImageFormat::from_path(path);
    // println!("ImageFormat : {:?}", format.unwrap());

    println!("DynamicImage variant: {:?}", image.color());

    let pixel = image.as_rgb32f().unwrap().get_pixel(1228, 385);
    println!("Pixel rgb 32f: {:?}", pixel.0);

    let image_rgba32f = image.to_rgba32f();
    let pixel = image_rgba32f.get_pixel(1228, 385);
    println!("Pixel rgba 32f: {:?}", pixel.0);
}
 */
use std::time::Instant;
use rayon::prelude::*;
use image::ImageReader;
use image::{GenericImageView, Pixel};
use stb_image::image::{load, LoadResult};

fn main() {
    // let path = "test.hdr";
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/core/clarens_night_02_2k.hdr"
    );
    
    // --- image-rs ---
    let dyn_img = ImageReader::open(path)
        .expect("Failed to open HDR file")
        .with_guessed_format()
        .expect("Failed to guess format")
        .decode()
        .expect("Failed to decode HDR");

    let (width, height) = dyn_img.dimensions();
    println!("image-rs size: {}x{}", width, height);

    // --- Serial version ---
    let start = Instant::now();
    let mut hdr_buffer_serial = Vec::with_capacity((width * height) as usize);
    for y in 0..height {
        for x in 0..width {
            let p = dyn_img.get_pixel(x, y).to_rgb();
            hdr_buffer_serial.push([p[0] as f32 / 255.0,
                                    p[1] as f32 / 255.0,
                                    p[2] as f32 / 255.0]);
        }
    }
    let duration = start.elapsed();
    println!("image-rs conversion to f32 (serial): {:.2?}", duration);
    println!("Total pixels: {}", hdr_buffer_serial.len());

    // --- Parallel version ---
    let start = Instant::now();
    let hdr_buffer_parallel: Vec<[f32; 3]> = (0..height as usize)
        .into_par_iter()
        .flat_map_iter(|y| {
            let dyn_img_ref = &dyn_img; // riferimento immutabile
            (0..width as usize).map(move |x| {
                let p = dyn_img_ref.get_pixel(x as u32, y as u32).to_rgb();
                [p[0] as f32 / 255.0, p[1] as f32 / 255.0, p[2] as f32 / 255.0]
            })
        })
        .collect();
    let duration = start.elapsed();
    println!("image-rs conversion to f32 (parallel): {:.2?}", duration);
    println!("Total pixels: {}", hdr_buffer_parallel.len());

    // --- stb_image ---
    let start = Instant::now();
    match load(path) {
        LoadResult::ImageF32(img) => {
            let duration = start.elapsed();
            println!("stb_image decode: {:.2?}", duration);
            println!(
                "stb_image size: {}x{} ({} channels)",
                img.width, img.height, img.depth
            );

            // Parallel processing in Vec<[f32;3]>
            let start = Instant::now();
            let processed: Vec<[f32; 3]> = img
                .data
                .par_chunks(img.depth)
                .map(|px| [px[0], px[1], px[2]])
                .collect();
            let duration = start.elapsed();
            println!("stb_image processing (parallel): {:.2?}", duration);
            println!("Total pixels: {}", processed.len());
        }
        LoadResult::ImageU8(img) => {
            println!("LDR image loaded: {}x{}x{}", img.width, img.height, img.depth);
        }
        LoadResult::Error(e) => {
            eprintln!("stb_image error: {}", e);
        }
    }
}
