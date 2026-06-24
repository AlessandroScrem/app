use std::collections::HashMap;

use crate::{
    assets::TextureId,
    gpu::static_textures,
    gpu::{
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

    #[allow(unused)]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn insert(&mut self, id: TextureId, texture: GpuTexture) {
        self.stats.add(texture.estimated_size);
        self.map.insert(id, texture);
    }

    pub fn get(&self, id: TextureId) -> Option<&GpuTexture> {
        self.map.get(&id)
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

    pub fn remove(&mut self, id: TextureId) {
        if let Some(gpu_texture) = self.map.remove(&id) {
            self.stats.remove(gpu_texture.estimated_size);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils;

    #[test]
    fn should_create_gpu_texture_cache() {
        let (device, queue) = test_utils::get_device_and_queue();
        let gpu_texture_cache = GpuTextureCache::new(device, queue);

        assert!(gpu_texture_cache.map.is_empty());
    }
}
