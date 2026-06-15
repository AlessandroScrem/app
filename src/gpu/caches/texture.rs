use std::collections::HashMap;

use crate::{
    assets::TextureId,
    gpu::static_textures,
    renderer::{
        GpuResourceStats, HasGpuStats,
        texture::{GpuTexture, GpuTextureBuilder},
    },
};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use wgpu::{Device, Queue};

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum CacheTextureSlot {
    White,
    Black,
    Normal,
}

pub struct GpuBuiltinTextures {
    builtin: Vec<GpuTexture>,
}

impl GpuBuiltinTextures {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let builtin: Vec<GpuTexture> = CacheTextureSlot::iter()
            .map(|slot| Self::create(device, queue, slot))
            .collect();

        Self { builtin }
    }

    pub fn get(&self, slot: CacheTextureSlot) -> &GpuTexture {
        &self.builtin[slot as usize]
    }

    fn create(device: &Device, queue: &Queue, slot: CacheTextureSlot) -> GpuTexture {
        match slot {
            CacheTextureSlot::White => {
                GpuTextureBuilder::from_static(&static_textures::WHITE_STATIC_TEXTURE)
                    .build(device, Some(queue))
            }
            CacheTextureSlot::Black => {
                GpuTextureBuilder::from_static(&static_textures::BLACK_STATIC_TEXTURE)
                    .build(device, Some(queue))
            }
            CacheTextureSlot::Normal => {
                GpuTextureBuilder::from_static(&static_textures::NORMAL_STATIC_TEXTURE)
                    .build(device, Some(queue))
            }
        }
    }
}

pub struct GpuTextureCache {
    map: HashMap<TextureId, GpuTexture>,
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
            map: HashMap::new(),
            stats: GpuResourceStats::default(),
            builtin,
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn insert(&mut self, id: TextureId, texture: GpuTexture) {
        self.stats.add(texture.estimated_size);
        self.map.insert(id, texture);
    }

    pub fn contain(&self, id: TextureId) -> bool {
        self.map.contains_key(&id)
    }

    pub fn get(&self, id: TextureId) -> &GpuTexture {
        &self.get_or(Some(id), CacheTextureSlot::White)
    }

    pub fn get_or(&self, id: Option<TextureId>, slot: CacheTextureSlot) -> &GpuTexture {
        id.and_then(|id| self.map.get(&id))
            .unwrap_or_else(|| self.builtin.get(slot))
    }

    pub fn contains_key(&self, id: &TextureId) -> bool {
        self.map.contains_key(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&TextureId, &GpuTexture)> {
        self.map.iter()
    }

    /*     fn retain<F>(&mut self, contains: F)
    where
        F: Fn(&TextureId) -> bool,
    {
        // Sync cleanup
        self.map.retain(|id, tex| {
            let keep = contains(&id);
            if !keep {
                // remove id
                // update stats
                self.stats.remove(tex.estimated_size);
                trace!("removed gpu tex {:?}", id);
            }
            keep
        });
    } */

    /*     #[allow(unused)]
    pub fn view(&self, id: TextureId) -> &wgpu::TextureView {
        &self.get_or_fallback_white(id).view
    }

    pub fn view_or(&self, id: Option<TextureId>, slot: CacheTextureSlot) -> &wgpu::TextureView {
        &id.and_then(|id| self.map.get(&id))
            .unwrap_or_else(|| self.builtin.get(slot))
            .view
    }

    */

    /*      pub fn upload_textures(
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
    }  */
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        assets::texture_asset::{SamplerDesc, TextureDesc, TextureUsage},
        test_utils,
    };
    const HDR_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/core/newport_loft.hdr");

    #[test]
    fn should_create_gpu_texture_cache() {
        let (device, queue) = test_utils::get_device_and_queue();
        let gpu_texture_cache = GpuTextureCache::new(device, queue);

        assert!(gpu_texture_cache.map.is_empty());
    }

    /*     #[test]
    fn should_load_hdr_texture_rgba32float() {
        let (device, queue) = test_utils::get_device_and_queue();
        let mut gpu_texture_cache = GpuTextureCache::new(device, queue);
        let mut texture_assets = TextureAssets::new();

        let desc = TextureDesc::File {
            path: HDR_PATH.into(),
            usage: TextureUsage::HDR32,
            sampler: SamplerDesc::LinearRepeat,
            mipmaps: false,
        };

        let texture_id = texture_assets.get_or_create(desc);

        texture_assets.load_cpu_textures();
        gpu_texture_cache.upload_textures(&mut texture_assets, device, queue);

        assert!(gpu_texture_cache.map.contains_key(texture_id));
    } */

    /*     #[test]
    fn should_contain_builtin() {
        let (device, queue) = test_utils::get_device_and_queue();
        let gpu_texture_cache = GpuTextureCache::new(&device, &queue);

        let _fallback_texture = gpu_texture_cache.get_or_fallback_white(TextureId::default());

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
    } */
}
