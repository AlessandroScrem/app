use super::static_textures::StaticTexture;
use crate::assets::{ColorSpace, SamplerDesc, texture_upload::TextureData};

use super::prelude::*;
use std::sync::Arc;

pub enum TextureSource<'a> {
    Cpu(TextureData),
    Static(&'a StaticTexture),
}

pub enum Dimension {
    D2,
    Cube,
}

pub struct GpuTexture {
    pub inner: Arc<wgpu::Texture>,
    pub view: Arc<wgpu::TextureView>,
    pub view_mips: Arc<wgpu::TextureView>,
    pub extent: wgpu::Extent3d,
    pub sampler: wgpu::Sampler,
    pub _format: wgpu::TextureFormat,
    pub estimated_size: usize,
}

pub struct GpuTextureBuilder<'a> {
    width: u32,
    height: u32,
    with_mips: Option<u32>,
    source: Option<TextureSource<'a>>,
    sampler: Option<SamplerDesc>,
    format: ColorSpace,
    dimension: Dimension,
    usage: GpuTextureUsage,
    label: Option<&'a str>,
}

pub enum GpuTextureUsage {
    EntityId,
    RenderTarget,
    DepthTarget,
    SampledTexture,
    SampledTextureStorage,
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
                wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC
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
            GpuTextureUsage::SampledTextureStorage => {
                wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::STORAGE_BINDING
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
            SamplerDesc::LinearMipmap => wgpu::SamplerDescriptor {
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
            dimension: Dimension::D2,
            usage: GpuTextureUsage::SampledTexture,
            sampler: Some(SamplerDesc::Linear),
            source: Some(TextureSource::Cpu(data)),
            with_mips: None,
        }
    }

    pub fn from_static(data: &'static StaticTexture) -> GpuTextureBuilder<'static> {
        GpuTextureBuilder {
            label: None,
            format: data.format.clone(),
            width: data.width,
            height: data.height,
            dimension: Dimension::D2,
            usage: GpuTextureUsage::SampledTexture,
            sampler: Some(SamplerDesc::Linear),
            source: Some(TextureSource::Static(data)),
            with_mips: None,
        }
    }

    pub fn from_empty(width: u32, height: u32) -> GpuTextureBuilder<'static> {
        GpuTextureBuilder {
            label: None,
            width,
            height,
            dimension: Dimension::D2,
            format: ColorSpace::Rgba8,
            usage: GpuTextureUsage::RenderTarget,
            sampler: None,
            source: None,
            with_mips: None,
        }
    }

    pub fn format(mut self, format: ColorSpace) -> Self {
        self.format = format;
        self
    }

    pub fn dimension(mut self, dimension: Dimension) -> Self {
        self.dimension = dimension;
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

    pub fn with_mips(mut self, max_mips: u32) -> Self {
        self.with_mips = Some(max_mips);
        self
    }
}

impl<'a> GpuTextureBuilder<'a> {
    pub fn build(self, device: &wgpu::Device, queue: Option<&wgpu::Queue>) -> GpuTexture {
        let (layers, view_dimension) = match self.dimension {
            Dimension::D2 => (1, wgpu::TextureViewDimension::D2),
            Dimension::Cube => (6, wgpu::TextureViewDimension::Cube),
        };

        let width = self.width;
        let height = self.height;

        let extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: layers,
        };

        let format = wgpu::TextureFormat::from(self.format);
        let usage = wgpu::TextureUsages::from(self.usage);

        let mip_level_count = if let Some(max_mips) = self.with_mips {
            use std::cmp::{max, min};
            // // lod calcuation based on texture size clamp to max_mips
            let mip_count = min(max_mips, u32::ilog2(max(width, height)));
            trace!("Texture with mips: {}", mip_count);
            mip_count
        } else {
            1
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: self.label,
            size: extent,
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });

        let mut estimated_size = 0;

        if let (Some(source), Some(queue)) = (self.source, queue) {
            let pixels = match source {
                TextureSource::Cpu(data) => data.pixels.to_vec(),
                TextureSource::Static(data) => data.pixels.to_vec(),
            };

            // let pixel_size = format.target_pixel_byte_cost().unwrap_or(4);
            let pixel_size = self.format.pixel_size();

            let face_size = (width * height * pixel_size) as usize;

            assert_eq!(
                pixels.len(),
                face_size,
                "Data pixels does not match: pixel_size * (w * h) "
            );

            // extend in case of Cube texture (layers = 6)
            let pixels: Vec<u8> = pixels
                .iter()
                .copied()
                .cycle()
                .take(face_size * layers as usize)
                .collect();

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

            // TODO: implement pixel size and with mips
            estimated_size = pixels.len();
        }

        // let estimated_size = (self.width * self.height * format.target_pixel_byte_cost().unwrap_or(4)) as usize;

        let _sampler = match self.sampler {
            Some(sd) => device.create_sampler(&sd.into()),
            None => device.create_sampler(&Default::default()),
        };

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(view_dimension),
            mip_level_count: Some(1),
            ..Default::default()
        });

        let view_mips = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(view_dimension),
            mip_level_count: None, // all mips
            ..Default::default()
        });

        GpuTexture {
            extent,
            inner: Arc::new(texture),
            view: Arc::new(view),
            view_mips: Arc::new(view_mips),
            sampler: _sampler,
            _format: format,
            estimated_size,
        }
    }
}
