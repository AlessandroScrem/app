use super::static_textures::StaticTexture;
use crate::assets::{ColorSpace, SamplerDesc, texture_upload::TextureData};

use std::sync::Arc;

pub enum TextureSource<'a> {
    Cpu(TextureData),
    Static(&'a StaticTexture),
}

pub struct GpuTexture {
    pub inner: Arc<wgpu::Texture>,
    pub view: Arc<wgpu::TextureView>,
    pub extent: wgpu::Extent3d,
    pub sampler: wgpu::Sampler,
    pub _format: wgpu::TextureFormat,
    pub estimated_size: usize,
}

pub struct GpuTextureBuilder<'a> {
    width: u32,
    height: u32,
    source: Option<TextureSource<'a>>,
    sampler: Option<SamplerDesc>,
    format: ColorSpace,
    usage: GpuTextureUsage,
    label: Option<&'a str>,
}

pub enum GpuTextureUsage {
    EntityId,
    RenderTarget,
    DepthTarget,
    SampledTexture,
}
impl From<GpuTextureUsage> for wgpu::TextureUsages {
    fn from(tu: GpuTextureUsage) -> Self {
        match tu {
            GpuTextureUsage::EntityId => {
                wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC
            }
            GpuTextureUsage::RenderTarget => {
                wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT
            }
            GpuTextureUsage::DepthTarget => {
                wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT
            }
            GpuTextureUsage::SampledTexture => {
                wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::RENDER_ATTACHMENT
            }
        }
    }
}
impl From<ColorSpace> for wgpu::TextureFormat {
    fn from(cs: ColorSpace) -> Self {
        match cs {
            ColorSpace::Rgba8 => wgpu::TextureFormat::Rgba8Unorm,
            ColorSpace::Srgba8 => wgpu::TextureFormat::Rgba8UnormSrgb,
            ColorSpace::Rgbaf16 => wgpu::TextureFormat::Rgba16Float,
            ColorSpace::Rgbaf32 => wgpu::TextureFormat::Rgba32Float,
            ColorSpace::Rg32ui => wgpu::TextureFormat::Rg32Uint,
            ColorSpace::Depth32f => wgpu::TextureFormat::Depth32Float,
        }
    }
}

impl From<SamplerDesc> for wgpu::SamplerDescriptor<'_> {
    fn from(sd: SamplerDesc) -> Self {
        match sd {
            SamplerDesc::Linear => wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            },
            SamplerDesc::Nearest => wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            },
        }
    }
}

impl<'a> GpuTextureBuilder<'a> {
    pub fn from_cpu(data: TextureData) -> GpuTextureBuilder<'static> {
        GpuTextureBuilder {
            label: None,
            format: data.format.clone(),
            width: data.width,
            height: data.height,
            usage: GpuTextureUsage::SampledTexture,
            sampler: Some(SamplerDesc::Linear),
            source: Some(TextureSource::Cpu(data)),
        }
    }

    pub fn from_static(data: &'static StaticTexture) -> GpuTextureBuilder<'static> {
        GpuTextureBuilder {
            label: None,
            format: data.format.clone(),
            width: data.width,
            height: data.height,
            usage: GpuTextureUsage::SampledTexture,
            sampler: Some(SamplerDesc::Linear),
            source: Some(TextureSource::Static(data)),
        }
    }

    pub fn from_empty(width: u32, height: u32) -> GpuTextureBuilder<'static> {
        GpuTextureBuilder {
            label: None,
            width,
            height,
            format: ColorSpace::Rgba8,
            usage: GpuTextureUsage::RenderTarget,
            sampler: None,
            source: None,
        }
    }

    pub fn format(mut self, format: ColorSpace) -> Self {
        self.format = format;
        self
    }

    pub fn usage(mut self, usage: GpuTextureUsage) -> Self {
        self.usage = usage;
        self
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn sampler(mut self, sampler: SamplerDesc) -> Self {
        self.sampler = Some(sampler);
        self
    }
}

impl<'a> GpuTextureBuilder<'a> {
    pub fn build(self, device: &wgpu::Device, queue: Option<&wgpu::Queue>) -> GpuTexture {
        let extent = wgpu::Extent3d {
            width: self.width,
            height: self.height,
            depth_or_array_layers: 1,
        };

        let format = wgpu::TextureFormat::from(self.format);
        let usage = wgpu::TextureUsages::from(self.usage);

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: self.label,
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });

        let mut estimated_size = 0;

        if let (Some(source), Some(queue)) = (self.source, queue) {
            let pixels: Vec<u8> = match source {
                TextureSource::Cpu(data) => data.pixels.to_vec(),
                TextureSource::Static(data) => data.pixels.to_vec(),
            };

            // let pixel_size = format.target_pixel_byte_cost().unwrap_or(4);
            let pixel_size = match self.format {
                ColorSpace::Rgba8 | ColorSpace::Srgba8 => 4,
                ColorSpace::Depth32f => 4,
                ColorSpace::Rgbaf16 => 8,
                ColorSpace::Rgbaf32 => 16,
                ColorSpace::Rg32ui => 8,
            };

            queue.write_texture(
                texture.as_image_copy(),
                &pixels,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(pixel_size * self.width),
                    rows_per_image: Some(self.height),
                },
                extent,
            );
            estimated_size = pixels.len();
        }

        // let estimated_size = (self.width * self.height * format.target_pixel_byte_cost().unwrap_or(4)) as usize;

        let _sampler = match self.sampler {
            Some(sd) => device.create_sampler(&sd.into()),
            None => device.create_sampler(&Default::default()),
        };

        let view = texture.create_view(&Default::default());

        GpuTexture {
            extent,
            inner: Arc::new(texture),
            view: Arc::new(view),
            sampler: _sampler,
            _format: format,
            estimated_size,
        }
    }
}
