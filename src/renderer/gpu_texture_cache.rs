use crate::assets::texture_upload::{CpuTexture, UploadPayload};

use super::*;
use slotmap::SecondaryMap;

#[derive(Default)]
pub struct GpuTextureCache {
    map: SecondaryMap<TextureId, GpuTexture>,
}

impl GpuTextureCache {
    pub fn get_or_fallback(
        &mut self,
        id: TextureId,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> &GpuTexture {
        self.map.entry(id).unwrap().or_insert_with(|| {
            info!("GpuTextureCache create Fallback Texture with id {:?}", id);
            GpuTexture::white_texture(device, queue)
        })
    }

    pub fn create_from_cpu(
        &mut self,
        id: TextureId,
        payload: UploadPayload,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> &GpuTexture {
        self.map
            .entry(id)
            .unwrap()
            .or_insert_with(|| GpuTexture::from_cpu(payload, device, queue))
    }

    pub fn retain(&mut self, assets: &TextureAssets) {
        // Sync cleanup
        self.map.retain(|id, _| assets.contains_key(id));
    }

    pub fn view(&self, id: TextureId) -> &wgpu::TextureView {
        &self
            .map
            .get(id)
            .expect("unable to get texture")
            .texture
            .view
    }

    pub fn contains_key(&self, id: &TextureId) -> bool {
        self.map.contains_key(*id)
    }

    pub fn remove(&mut self, id: &TextureId) {
        if self.map.contains_key(*id) {
            self.map.remove(*id);
        }
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
                trace!("Upload texture {:?} {:?} to gpu", id, asset.state);
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

pub struct GpuTexture {
    pub texture: texture::Texture,
}
impl GpuTexture {
    pub fn white_texture(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let white_bytes = CpuTexture {
            width: 1,
            height: 1,
            format: ColorSpace::Rgba8,
            pixels: vec![255, 255, 255, 255],
        };

        let texture = texture::Texture::from_cpu(&device, &queue, &white_bytes);
        Self { texture }
    }

    fn from_cpu(payload: UploadPayload, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        match payload {
            UploadPayload::Ready(cpu) => Self {
                texture: texture::Texture::from_cpu(device, queue, &cpu),
            },
            UploadPayload::Fallback => Self::white_texture(device, queue),
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
        let gpu_texture_cache = GpuTextureCache::default();

        assert!(gpu_texture_cache.map.is_empty());
    }

    #[test]
    fn should_load_hdr_texture_rgba32float() {
        let (device, queue) = test_utils::get_device_and_queue();
        let mut gpu_texture_cache = GpuTextureCache::default();
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
}
