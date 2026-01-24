pub mod bbox_manager;
pub mod gpu_manager;
pub mod gpu_renderer;
pub mod hdr_frame;
pub mod light_manager;
pub mod mesh_manager;
pub mod pipeline_manager;
pub mod renderpass;
pub mod skybox_manager;
pub mod uniform;

pub use gpu_renderer::Renderer;

pub use bbox_manager::BBoxManager;
pub use gpu_manager::GpuManager;
pub use gpu_renderer::{GpuDevice, ImGuiTextureRegistry};
pub use hdr_frame::{HdrFrame, IDTexture};
pub use light_manager::LightManager;
pub use mesh_manager::MeshManager;
pub use pipeline_manager::PipelineManager;
pub use skybox_manager::SkyboxManager;
pub use uniform::{CameraUniform, GlobalUniform, LightUniform};

use crate::assets::{ColorSpace, TextureAssets, TextureDesc, TextureId, file, texture};
use std::collections::HashMap;



pub static WHITE_TEXTURE_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/core/white.png"
));

pub struct GpuTexture {
    pub texture: texture::Texture,
}

#[derive(Default)]
pub struct GpuTextureCache {
    map: HashMap<TextureId, GpuTexture>,
}

impl GpuTextureCache {
    pub fn get_or_create(
        &mut self,
        id: TextureId,
        assets: &TextureAssets,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> &GpuTexture {
        self.map.entry(id).or_insert_with(|| {
            let desc = assets.storage.get(id).unwrap();
            GpuTexture::from_desc(desc, device, queue)
        })
    }
}

impl From<ColorSpace> for wgpu::TextureFormat {
    fn from(cs: ColorSpace) -> Self {
        match cs {
            ColorSpace::Rgba8 => wgpu::TextureFormat::Rgba8Unorm,
            ColorSpace::Srgba8 => wgpu::TextureFormat::Rgba8UnormSrgb,
            ColorSpace::Rgbaf16 => wgpu::TextureFormat::Rgba16Float,
            ColorSpace::Rgbaf32 => wgpu::TextureFormat::Rgba32Float,
        }
    }
}

impl GpuTexture {
    fn from_embedded_png(desc: &TextureDesc, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let texture = texture::Texture::new(
            &device,
            &queue,
            WHITE_TEXTURE_BYTES,
            desc.key.color_space.into(),
        );
        Self { texture }
    }

    fn from_desc(desc: &TextureDesc, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        match file::read_bytes(desc.key.path.as_path()) {
            Some(buffer) => Self {
                texture: texture::Texture::new(device, queue, &buffer, desc.key.color_space.into()),
            },
            None => Self::from_embedded_png(desc, device, queue),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{assets::{SamplerDesc, TextureKey}, test_utils};
    const HDR_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/core/newport_loft.hdr");
    
    #[test]
    fn should_create_gpu_texture_cache() {
        let gpu_texture_cache = GpuTextureCache::default();
        
        assert!(gpu_texture_cache.map.is_empty());
    }    
    
    #[test]
    fn should_load_hdr_texture_rgba32float() {
        let (device, queue) = test_utils::get_device_and_queue();
        let mut gpu_texture_cache = GpuTextureCache::default();
        let mut texture_assets = TextureAssets::new();

        let key = TextureKey{path: HDR_PATH.into(), color_space: ColorSpace::Rgbaf32, usage: crate::assets::TextureUsage::HDR32};
        let desc = TextureDesc{key, sampler: SamplerDesc::Linear, mipmaps: false};

        let texture_id = texture_assets.get_or_create(desc);

        let _ = gpu_texture_cache.get_or_create(texture_id, &texture_assets, device, queue);

        assert!(gpu_texture_cache.map.contains_key(&texture_id) );
    }
}

