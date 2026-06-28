use crate::{
    assets::texture_asset::{ColorSpace, SamplerDesc},
    gpu::{GpuTexture, GpuTextureBuilder, GpuTextureUsage},
};

pub struct ShadowManager {
    texture: GpuTexture,
}

impl ShadowManager {
    pub fn new(device: &wgpu::Device, size: u32) -> Self {
        let texture = GpuTextureBuilder::from_empty(size, size)
            .format(ColorSpace::Depth32f)
            .usage(GpuTextureUsage::SampledTexture)
            .sampler(SamplerDesc::NearestClamp)
            .label("depth_texture")
            .build(device, None);

        Self { texture }
    }
}
