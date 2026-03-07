use crate::{
    assets::texture_upload::UploadPayload,
    renderer::texture::{GpuTexture, GpuTextureBuilder},
};

use super::*;
use slotmap::SecondaryMap;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use wgpu::{Device, Queue};

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum TextureSlot {
    White,
    Normal,
}

struct GpuBuiltinTextures {
    builtin: Vec<GpuTexture>,
}

impl GpuBuiltinTextures {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let builtin: Vec<GpuTexture> = TextureSlot::iter()
            .map(|slot| Self::create(device, queue, slot))
            .collect();

        Self { builtin }
    }

    fn get(&self, slot: TextureSlot) -> &GpuTexture {
        &self.builtin[slot as usize]
    }

    fn create(device: &Device, queue: &Queue, slot: TextureSlot) -> GpuTexture {
        match slot {
            TextureSlot::White => {
                GpuTextureBuilder::from_static(&static_textures::WHITE_STATIC_TEXTURE)
                    .build(device, queue)
            }
            TextureSlot::Normal => {
                GpuTextureBuilder::from_static(&static_textures::NORMAL_STATIC_TEXTURE)
                    .build(device, queue)
            }
        }
    }
}

pub struct GpuTextureCache {
    map: SecondaryMap<TextureId, GpuTexture>,
    builtin: GpuBuiltinTextures,
    stats: GpuResourceStats,
}

impl HasGpuStats for GpuTextureCache {
    fn get_stats(&self) -> GpuResourceStats {
        self.stats.clone()
    }
}
impl GpuTextureCache {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let builtin = GpuBuiltinTextures::new(device, queue);

        Self {
            map: SecondaryMap::new(),
            stats: GpuResourceStats::default(),
            builtin,
        }
    }

    pub fn get_or_fallback(&self, id: TextureId) -> &GpuTexture {
        self.map
            .get(id)
            .unwrap_or(self.builtin.get(TextureSlot::White))
    }

    fn create_from_cpu(
        &mut self,
        id: TextureId,
        payload: UploadPayload,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        match payload {
            UploadPayload::Ready(data) => {
                let texture = GpuTextureBuilder::from_cpu(data).build(device, queue);
                self.map.insert(id, texture);
            }
            UploadPayload::Fallback => {}
        }
    }

    pub fn retain(&mut self, assets: &TextureAssets) {
        // Sync cleanup
        self.map.retain(|id, tex| {
            if assets.contains_key(id) {
                true //mantain
            } else {
                // update stats
                self.stats.remove(tex.estimated_size);
                trace!("removed gpu tex {:?}", id);
                false // remove
            }
        });
    }

    pub fn view(&self, id: TextureId) -> &wgpu::TextureView {
        &self.get_or_fallback(id).view
    }

    pub fn contains_key(&self, id: &TextureId) -> bool {
        self.map.contains_key(*id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (TextureId, &GpuTexture)> {
        self.map.iter()
    }

    pub fn upload_textures(
        &mut self,
        source: &mut impl texture_upload::TextureUploadSource,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let dirty = source.drain_dirty_textures();

        for (id, cpu_texture) in dirty {
            if let Some(asset) = source.get_texture_asset(id) {
                self.create_from_cpu(id, cpu_texture, device, queue);
                trace!("Gpu Upload texture {:?} {:?} ", id, asset.state);
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        assets::{SamplerDesc, TextureKey},
        test_utils,
    };
    const HDR_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/core/newport_loft.hdr");

    #[test]
    fn should_create_gpu_texture_cache() {
        let (device, queue) = test_utils::get_device_and_queue();
        let gpu_texture_cache = GpuTextureCache::new(device, queue);

        assert!(gpu_texture_cache.map.is_empty());
    }

    #[test]
    fn should_load_hdr_texture_rgba32float() {
        let (device, queue) = test_utils::get_device_and_queue();
        let mut gpu_texture_cache = GpuTextureCache::new(device, queue);
        let mut texture_assets = TextureAssets::new();

        let key = TextureKey::File {
            path: HDR_PATH.into(),
            color_space: ColorSpace::Rgbaf32,
            usage: crate::assets::TextureUsage::HDR32,
        };
        let desc = TextureDesc::File {
            key,
            sampler: SamplerDesc::Linear,
            mipmaps: false,
        };

        let texture_id = texture_assets.get_or_create(desc);

        texture_assets.load_cpu_textures();
        gpu_texture_cache.upload_textures(&mut texture_assets, device, queue);

        assert!(gpu_texture_cache.map.contains_key(texture_id));
    }

    #[test]
    fn should_contain_builtin() {
        let (device, queue) = test_utils::get_device_and_queue();
        let gpu_texture_cache = GpuTextureCache::new(&device, &queue);

        let _fallback_texture = gpu_texture_cache.get_or_fallback(TextureId::default());

        #[cfg(feature = "save_tests")]
        {
            test_utils::save_texture(
                device,
                queue,
                "fallback_texture.png",
                &_fallback_texture.inner,
                0,
            )
            .unwrap();
            test_utils::save_texture(
                device,
                queue,
                "fallback_texture.png",
                &_fallback_texture.inner,
                0,
            )
            .unwrap();
        }
    }
}
