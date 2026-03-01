use crate::{
    assets::texture_upload::{CpuTexture, UploadPayload, load_cpu_textures_par},
    renderer::texture::GpuTexture,
};

use super::*;
use slotmap::SecondaryMap;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use wgpu::{Device, Queue};

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum TextureSlot {
    LightIcon,
}

pub struct GpuTextureCache {
    map: SecondaryMap<TextureId, GpuTexture>,
    builtin: Vec<GpuTexture>,
    stats: GpuResourceStats,
}

impl HasGpuStats for GpuTextureCache {
    fn get_stats(&self) -> GpuResourceStats {
        self.stats.clone()
    }
}
impl GpuTextureCache {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let builtin: Vec<GpuTexture> = TextureSlot::iter()
            .map(|slot| Self::create_builtin(device, queue, slot))
            .collect();

        Self {
            map: SecondaryMap::new(),
            stats: GpuResourceStats::default(),
            builtin,
        }
    }
    pub fn get_builtin(&self, slot: TextureSlot) -> &GpuTexture {
        &self.builtin[slot as usize]
    }

    pub fn get_or_fallback(
        &mut self,
        id: TextureId,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> &GpuTexture {
        self.map.entry(id).unwrap().or_insert_with(|| {
            info!("GpuTextureCache create Fallback Texture with id {:?}", id);
            let texture = GpuTexture::white_texture(device, queue);
            self.stats.add(texture.estimated_size);
            texture
        })
    }

    pub fn create_from_cpu(
        &mut self,
        id: TextureId,
        payload: UploadPayload,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> &GpuTexture {
        self.map.entry(id).unwrap().or_insert_with(|| {
            let texture = GpuTexture::from_cpu(payload, device, queue);
            self.stats.add(texture.estimated_size);
            texture
        })
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
        &self.map.get(id).expect("unable to get texture").view
    }

    pub fn contains_key(&self, id: &TextureId) -> bool {
        self.map.contains_key(*id)
    }

    pub fn ensure(&mut self, id: TextureId, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.get_or_fallback(id, device, queue);
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

impl GpuTextureCache {
    fn create_builtin(device: &Device, queue: &Queue, slot: TextureSlot) -> GpuTexture {
        match slot {
            TextureSlot::LightIcon => {
                static LIGHT_BULB_BYTES: &[u8] = include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/assets/core/lightbulb-icon32.png"
                ));
                let (pixels, width, height) = image_decoder::decode_stb_image_rgaba8(
                    &LIGHT_BULB_BYTES,
                )
                .unwrap_or_else(|_| {
                    (
                        CpuTexture::white().pixels,
                        CpuTexture::white().width,
                        CpuTexture::white().height,
                    )
                });
                let cpu_data = CpuTexture {
                    pixels,
                    width,
                    height,
                    format: ColorSpace::Rgba8,
                };
                GpuTexture::from_cpu_texture(device, queue, cpu_data)
            }
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

        let _texture = gpu_texture_cache.get_builtin(TextureSlot::LightIcon);

        #[cfg(feature = "save_tests")]
        test_utils::save_texture(device, queue, "texture.png", &_texture.inner, 0).unwrap()
    }
}
