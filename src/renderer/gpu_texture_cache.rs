
use super::*;
use slotmap::SecondaryMap;

pub static WHITE_TEXTURE_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/core/white.png"
));

#[derive(Default)]
pub struct GpuTextureCache {
    map: SecondaryMap<TextureId, GpuTexture>,
}

impl GpuTextureCache {
    pub fn get_or_create(
        &mut self,
        id: TextureId,
        assets: &TextureAssets,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> &GpuTexture {
        
        self.map.entry(id).unwrap().or_insert_with(|| {
            let desc = assets.storage.get(id).unwrap();
            GpuTexture::from_desc(desc, device, queue)
        })
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

    pub fn contains_key(&self, id: &TextureId) ->bool {
        self.map.contains_key(*id)
    }

    pub fn remove(&mut self, id: &TextureId) {
        if self.map.contains_key(*id) {
            self.map.remove(*id);
        }
    }

    pub fn ensure(
        &mut self,
        id: TextureId,
        assets: &TextureAssets,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        self.get_or_create(id, assets, device, queue);
    }

    pub fn iter(&self) -> impl Iterator<Item = (TextureId, &GpuTexture)> {
        self.map.iter()
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
        let texture = texture::Texture::new(
            &device,
            &queue,
            WHITE_TEXTURE_BYTES,
            wgpu::TextureFormat::Rgba8Unorm,
        );
        Self { texture }
    }

    fn from_desc(desc: &TextureDesc, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        match desc {
            TextureDesc::White => return Self::white_texture(device, queue),
            TextureDesc::File {
                key,
                sampler: _,
                mipmaps: _,
            } => match key {
                TextureKey::File {
                    path,
                    color_space,
                    usage: _,
                } => {
                    let buffer = file::read_bytes(path).expect("Failed to read texture file");
                    Self {
                        texture: texture::Texture::new(
                            device,
                            queue,
                            &buffer,
                            (*color_space).into(),
                        ),
                    }
                }
                TextureKey::White => Self::white_texture(device, queue),
            },
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

        let _ = gpu_texture_cache.get_or_create(texture_id, &texture_assets, device, queue);

        assert!(gpu_texture_cache.map.contains_key(texture_id));
    }
}
