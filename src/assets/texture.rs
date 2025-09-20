use stb_image::image::load_from_memory_with_depth;
use std::sync::Arc;
use wgpu::TextureFormat;

fn decode_stb_image_par(buffer: &[u8]) -> (Vec<u8>, u32, u32) {
    use half::f16;
    use rayon::prelude::*;
    use stb_image::image::LoadResult;
    // Caricamento HDR con stb_image

    let img = match load_from_memory_with_depth(buffer, 4, false) {
        LoadResult::ImageF32(img) => img,
        LoadResult::Error(e) => panic!("Failed: {}", e),
        _ => panic!("stb_image: Unexpected format"),
    };

    let width = img.width as u32;
    let height = img.height as u32;
    let num_pixels = (width * height) as usize;

    let timer = std::time::Instant::now();
     // Prealloca il buffer finale: 4 canali * 2 byte per pixel
    let mut raw_u8 = vec![0u8; num_pixels * 4 * 2];

    // Parallel map diretto: pixel source -> pixel destination
    raw_u8
        .par_chunks_mut(8)                 // 8 byte per pixel RGBA16
        .zip(img.data.par_chunks(4))       // 4 float per pixel RGBA, passiamo un reference
        .for_each(|(dst, src)| {
            dst[0..2].copy_from_slice(&f16::from_f32(src[0].clamp(0.0, f16::MAX.to_f32())).to_le_bytes());
            dst[2..4].copy_from_slice(&f16::from_f32(src[1].clamp(0.0, f16::MAX.to_f32())).to_le_bytes());
            dst[4..6].copy_from_slice(&f16::from_f32(src[2].clamp(0.0, f16::MAX.to_f32())).to_le_bytes());
            dst[6..8].copy_from_slice(&f16::from_f32(src[3].clamp(0.0, f16::MAX.to_f32())).to_le_bytes());
        });

    println!(
        "Time for decoding HDR (stb_image, parallel): {:?}",
        timer.elapsed().as_millis()
    );
    (raw_u8, width, height)
}

fn read_stb_image(buffer: &[u8]) -> (Vec<u8>, u32, u32) {

    use stb_image::image::LoadResult;
    // Caricamento LDR con stb_image

    let timer = std::time::Instant::now();
    let img = match load_from_memory_with_depth(buffer, 4, false) {
        LoadResult::ImageU8(img) => img,
        LoadResult::Error(e) => panic!("Failed: {}", e),
        _ => panic!("stb_image: Unexpected format"),
    };

    let width = img.width as u32;
    let height = img.height as u32;

    let raw_u8 = img.data;

    println!(
        "Time for Load LDR image (stb_image, parallel): {:?}",
        timer.elapsed().as_millis()
    );
    (raw_u8, width, height)
}


pub struct Texture {
    pub inner: Arc<wgpu::Texture>,
    pub view: Arc<wgpu::TextureView>,
    pub extent: wgpu::Extent3d,
    pub _format: TextureFormat,
}

impl Texture {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        buffer: &[u8],
        format: TextureFormat,
    ) -> Self {
        let (raw_data, width, height, pixel_size) = match format {
            TextureFormat::Rgba8UnormSrgb | TextureFormat::Rgba8Unorm => {
                // image_rs well optimized in release mode
                // let image = image::load_from_memory(buffer).unwrap().to_rgba8();
                // let (width, height) = image.dimensions();
                // let raw = image.into_raw(); // già Vec<u8>
                let (raw, width, height) = read_stb_image(buffer);
                (raw, width, height, 4)
            }
            // formato non compatibile con imgui perchè non filterable
            TextureFormat::Rgba32Float => {
                let image = image::load_from_memory(buffer).unwrap().to_rgba32f();
                let (width, height) = image.dimensions();
                let raw_f32: Vec<f32> = image.into_raw();
                let raw_u8: Vec<u8> = bytemuck::cast_slice(&raw_f32).to_vec();
                (raw_u8, width, height, 16)
            }
            TextureFormat::Rgba16Float => {
                let (raw_u8, width, height) = decode_stb_image_par(buffer);
                (raw_u8, width, height, 8)
            }
            _ => panic!("Unsopported TextureFormat"),
        };

        let extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &raw_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(pixel_size * width),
                rows_per_image: Some(height),
            },
            extent,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Texture {
            extent,
            inner: Arc::new(texture),
            view: Arc::new(view),
            _format: format,
        }
    }
}

pub struct CubeTexture {
    pub inner: Arc<wgpu::Texture>,
    pub view: Arc<wgpu::TextureView>,
    pub extent: wgpu::Extent3d,
    _format: TextureFormat,
}

impl CubeTexture {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        buffers: &[&[u8]],
        format: TextureFormat,
    ) -> Self {
        let (images, pixel_size) = match format {
            TextureFormat::Rgba8UnormSrgb | TextureFormat::Rgba8Unorm => {
                let mut images = Vec::new();
                for buffer in buffers {
                    images.push(image::load_from_memory(buffer).unwrap().to_rgba8());
                }
                (images, 4)
            }
            TextureFormat::Rgba16Float => {
                unimplemented!("Rgba16Float not yet implemented for CubeTexture")
            }
            _ => panic!("Unsopported TextureFormat"),
        };

        let (width, height) = images[0].dimensions();
        assert!(width == height, "Image size not a square");
        for image in &images {
            assert_eq!(
                image.dimensions(),
                (width, height),
                "Cube has different sizes"
            )
        }

        let extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 6, // <- 6 faces,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Cubemap"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        for (i, face) in images.iter().enumerate() {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    aspect: wgpu::TextureAspect::All,
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: i as u32,
                    },
                },
                face,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(pixel_size * width),
                    rows_per_image: Some(height),
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1, // <- 1 face,
                },
            );
        }

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });

        CubeTexture {
            extent,
            inner: Arc::new(texture),
            view: Arc::new(view),
            _format: format,
        }
    }
}
