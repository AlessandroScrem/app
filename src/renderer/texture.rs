use crate::{assets::texture_upload::CpuTexture, prelude::*};
use std::sync::Arc;
use wgpu::{TextureAspect, TextureDimension, TextureFormat, TextureUsages};

pub struct Texture {
    pub inner: Arc<wgpu::Texture>,
    pub view: Arc<wgpu::TextureView>,
    pub extent: wgpu::Extent3d,
    pub _format: TextureFormat,
}

impl Texture {
    pub fn from_cpu(device: &wgpu::Device, queue: &wgpu::Queue, cpu_data: &CpuTexture) -> Self {
        let width = cpu_data.width;
        let height = cpu_data.height;
        let format: wgpu::TextureFormat = cpu_data.format.into();
        let raw_data = &cpu_data.pixels;

        let pixel_size = match cpu_data.format {
            assets::ColorSpace::Rgba8 | assets::ColorSpace::Srgba8 => 4,
            assets::ColorSpace::Rgbaf16 => 8,
            assets::ColorSpace::Rgbaf32 => 16,
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
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: TextureAspect::All,
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
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        for (i, face) in images.iter().enumerate() {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    aspect: TextureAspect::All,
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
