use crate::{
    assets::texture_upload::{CpuTexture, UploadPayload},
    prelude::*,
};
use std::sync::Arc;
use wgpu::{TextureAspect, TextureDimension, TextureFormat, TextureUsages};

pub(crate) struct GpuTexture {
    pub(crate) inner: Arc<wgpu::Texture>,
    pub(crate) view: Arc<wgpu::TextureView>,
    pub(crate) extent: wgpu::Extent3d,
    pub(crate) _format: TextureFormat,
}

impl GpuTexture {
    pub(crate) fn from_cpu(payload: UploadPayload, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        match payload {
            UploadPayload::Ready(cpu) => Self::from_cpu_texture(device, queue, cpu),
            UploadPayload::Fallback => Self::white_texture(device, queue),
        }
    }

    pub(crate) fn white_texture(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        Self::from_cpu_texture(&device, &queue, CpuTexture::white())
    }

    fn from_cpu_texture(device: &wgpu::Device, queue: &wgpu::Queue, cpu_data: CpuTexture) -> Self {
        let width = cpu_data.width;
        let height = cpu_data.height;
        let format = wgpu::TextureFormat::from(cpu_data.format);
        let pixels = &cpu_data.pixels;

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
            &pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(pixel_size * width),
                rows_per_image: Some(height),
            },
            extent,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            extent,
            inner: Arc::new(texture),
            view: Arc::new(view),
            _format: format,
        }
    }
}
