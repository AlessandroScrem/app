use super::static_textures::StaticTexture;
use crate::{
    assets::texture_upload::TextureData,
    prelude::*,
};
use std::sync::Arc;
use wgpu::TextureFormat;

pub enum TextureSource<'a> {
    Cpu(TextureData),
    Static(&'a StaticTexture),
}

pub struct GpuTextureBuilder<'a> {
    source: TextureSource<'a>,
    sampler: Option<wgpu::SamplerDescriptor<'a>>,
    label: Option<&'a str>,
}

impl<'a> GpuTextureBuilder<'a> {
    fn from_source(source: TextureSource<'a>) -> Self {
        Self {
            source,
            sampler: None,
            label: None,
        }
    }

    pub fn from_cpu(data: TextureData) -> GpuTextureBuilder<'static> {
        GpuTextureBuilder {
            source: TextureSource::Cpu(data),
            label: None,
            sampler: None,
        }
    }

    pub fn from_static(tex: &'static StaticTexture) -> GpuTextureBuilder<'static> {
        GpuTextureBuilder {
            source: TextureSource::Static(tex),
            label: None,
            sampler: None,
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn sampler(mut self, sampler: wgpu::SamplerDescriptor<'a>) -> Self {
        self.sampler = Some(sampler);
        self
    }
}

impl<'a> GpuTextureBuilder<'a> {
    pub fn build(self, device: &wgpu::Device, queue: &wgpu::Queue) -> GpuTexture {
        let (width, height, pixels, format) = match self.source {
            TextureSource::Cpu(data) => (data.width, data.height, data.pixels, data.format),
            TextureSource::Static(tex) => (tex.width, tex.height, tex.pixels.to_vec(), tex.format),
        };

        let pixel_size = match format {
            assets::ColorSpace::Rgba8 | assets::ColorSpace::Srgba8 => 4,
            assets::ColorSpace::Rgbaf16 => 8,
            assets::ColorSpace::Rgbaf32 => 16,
        };

        let format = wgpu::TextureFormat::from(format);

        let extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: self.label,
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        queue.write_texture(
            texture.as_image_copy(),
            &pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(pixel_size * width),
                rows_per_image: Some(height),
            },
            extent,
        );

        // let sampler = device.create_sampler(
        //     &self.sampler.unwrap_or(wgpu::SamplerDescriptor::default())
        // );

        let view = texture.create_view(&Default::default());
        let estimated_size = pixels.len();

        GpuTexture {
            extent,
            inner: Arc::new(texture),
            view: Arc::new(view),
            _format: format,
            estimated_size,
        }
    }
}

pub struct GpuTexture {
    pub inner: Arc<wgpu::Texture>,
    pub view: Arc<wgpu::TextureView>,
    pub extent: wgpu::Extent3d,
    pub _format: TextureFormat,
    pub estimated_size: usize,
}
