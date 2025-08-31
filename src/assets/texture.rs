use std::sync::Arc;
use wgpu::TextureFormat;

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
        // assert!(width != 0 && height != 0, "image: dimension must > 0");

        let (raw_data, width, height, pixel_size) = match format {
            TextureFormat::Rgba8UnormSrgb | TextureFormat::Rgba8Unorm => {
                let image = image::load_from_memory(buffer).unwrap().to_rgba8();
                let (width, height) = image.dimensions();
                let raw = image.into_raw(); // già Vec<u8>
                (raw, width, height, 4)
            }
            // formato non compatibile con imgui perchè non filterable
            TextureFormat::Rgba32Float => {
                let image = image::load_from_memory(buffer).unwrap().to_rgba32f();
                let (width, height) = image.dimensions();
                let raw_f32: Vec<f32> = image.into_raw();
                let raw_u8: Vec<u8> = bytemuck::cast_slice(&raw_f32).to_vec(); // ✅ copia sicura in Vec<u8>
                (raw_u8, width, height, 16)
            }
            TextureFormat::Rgba16Float => {
                let image = image::load_from_memory(buffer).unwrap().to_rgba32f();
                let (width, height) = image.dimensions();
                let raw_f32: Vec<f32> = image.into_raw();
                // conversione diretta in Vec<u8> per Rgba16Float
                let raw_u8: Vec<u8> = raw_f32
                    .iter()
                    .flat_map(|f| half::f16::from_f32(*f).to_le_bytes())
                    .collect();
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
            TextureFormat::Rgba16Float => unimplemented!(),
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
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
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
