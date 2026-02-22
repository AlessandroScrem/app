use crate::assets::texture_upload::{TextureUploadSource, UploadPayload};

use super::*;
use std::cell::Cell;

impl TextureId {
    pub fn white(assets: &TextureAssets) -> TextureId {
        assets.white()
    }
}

// Textures
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum ColorSpace {
    Rgbaf32,
    Rgbaf16,
    Srgba8,
    Rgba8,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum TextureUsage {
    Albedo,
    Normal,
    MetallicRoughness,
    Emissive,
    Occlusion,
    HDR16,
    HDR32,
}

impl From<material_asset::MaterialTextureSlot> for TextureUsage {
    fn from(slot: material_asset::MaterialTextureSlot) -> Self {
        match slot {
            material_asset::MaterialTextureSlot::BaseColor => TextureUsage::Albedo,
            material_asset::MaterialTextureSlot::Normal => TextureUsage::Normal,
            material_asset::MaterialTextureSlot::MetallicRoughness => {
                TextureUsage::MetallicRoughness
            }
            material_asset::MaterialTextureSlot::Emissive => TextureUsage::Emissive,
            material_asset::MaterialTextureSlot::Occlusion => TextureUsage::Occlusion,
        }
    }
}

impl TextureUsage {
    pub fn color_space(self) -> ColorSpace {
        match self {
            TextureUsage::Albedo | TextureUsage::Emissive => ColorSpace::Srgba8,
            TextureUsage::Normal | TextureUsage::Occlusion | TextureUsage::MetallicRoughness => {
                ColorSpace::Rgba8
            }
            TextureUsage::HDR16 => ColorSpace::Rgbaf16,
            TextureUsage::HDR32 => ColorSpace::Rgbaf32,
        }
    }
}

impl From<wgpu::TextureFormat> for ColorSpace {
    fn from(format: wgpu::TextureFormat) -> Self {
        match format {
            wgpu::TextureFormat::Rgba8Unorm => ColorSpace::Rgba8,
            wgpu::TextureFormat::Rgba8UnormSrgb => ColorSpace::Srgba8,
            wgpu::TextureFormat::Rgba16Float => ColorSpace::Rgbaf16,
            wgpu::TextureFormat::Rgba32Float => ColorSpace::Rgbaf32,
            _ => unimplemented!(),
        }
    }
}

#[derive(Default, Clone, Debug)]
pub enum SamplerDesc {
    #[default]
    Linear,
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum TextureKey {
    File {
        path: PathBuf,
        color_space: ColorSpace,
        usage: TextureUsage,
    },
    White,
}

#[derive(Clone, Debug)]
pub enum TextureDesc {
    File {
        key: TextureKey,
        sampler: SamplerDesc,
        mipmaps: bool,
    },
    White,
}

#[derive(Clone, PartialEq, Debug)]
pub struct TextureInfo {
    pub width: u32,
    pub height: u32,
    pub format: ColorSpace,
    pub byte_size: usize,
}

#[derive(Clone, PartialEq, Debug)]
pub enum TextureState {
    MetaOnly,              // solo path noto
    CpuReady(TextureInfo), // cpu texture caricata
    Fallback,              // errore → fallback
}

#[derive(Clone)]
pub struct TextureAsset {
    pub state: TextureState,
    desc: TextureDesc,
    ref_count: Cell<u32>,
}

impl TextureUploadSource for TextureAssets {
    fn drain_dirty_textures(&mut self) -> Vec<(TextureId, UploadPayload)> {
        std::mem::take(&mut self.dirty_textures)
    }
    
    fn get_texture_asset(&self, id: TextureId) -> Option<&TextureAsset> {
        self.storage.get(id)
    }
}

pub struct TextureAssets {
    storage: SlotMap<TextureId, TextureAsset>,
    lookup: HashMap<TextureKey, TextureId>,
    white: TextureId,
    dirty_textures: Vec<(TextureId, UploadPayload)>,
}

impl Default for TextureAssets {
    fn default() -> Self {
        let mut storage = SlotMap::with_key();
        let mut lookup = HashMap::new();
        let white_key = TextureKey::White;
        let white_id = storage.insert(TextureAsset {
            state: TextureState::MetaOnly,
            desc: TextureDesc::White,
            ref_count: Cell::new(1),
        });

        lookup.insert(white_key, white_id);

        Self {
            storage,
            lookup,
            white: white_id,
            dirty_textures: Vec::new(),
        }
    }
}

impl TextureAssets {
    pub fn new() -> Self {
        TextureAssets::default()
    }

    pub fn white(&self) -> TextureId {
        self.white
    }

    pub fn get_desc(&self, id: TextureId) -> Option<&TextureDesc> {
        self.storage.get(id).map(|t| &t.desc)
    }

    pub fn contains_key(&self, id: TextureId) -> bool {
        self.storage.contains_key(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (TextureId, &TextureDesc)> {
        self.storage.iter().map(|(id, asset)| (id, &asset.desc))
    }

    pub fn remove(&mut self, id: TextureId) {
        // Do not remove White!
        if id == self.white {
            return;
        }

        if let Some(asset) = self.storage.get(id) {
            let count = asset.ref_count.get();

            if count > 1 {
                asset.ref_count.set(count - 1);
            } else {
                let removed = self.storage.remove(id).unwrap();
                if let TextureDesc::File { key, .. } = removed.desc {
                    self.lookup.remove(&key);
                }
            }
        }
    }

    pub fn get_or_create(&mut self, desc: TextureDesc) -> TextureId {
        match desc {
            TextureDesc::White => self.white,

            TextureDesc::File {
                key,
                sampler,
                mipmaps,
            } => match self.lookup.get(&key) {
                Some(&id) => {
                    let tex = &self.storage[id];
                    tex.ref_count.set(tex.ref_count.get() + 1);
                    id
                }

                None => {
                    let id = self.storage.insert(TextureAsset {
                        state: TextureState::MetaOnly,
                        desc: TextureDesc::File {
                            key: key.clone(),
                            sampler,
                            mipmaps,
                        },
                        ref_count: Cell::new(1),
                    });
                    self.lookup.insert(key, id);
                    id
                }
            },
        }
    }

    pub fn from_file(&mut self, path: impl Into<PathBuf>, usage: TextureUsage) -> TextureId {
        let key = TextureKey::File {
            color_space: usage.color_space(),
            path: path.into(),
            usage,
        };

        let desc = TextureDesc::File {
            key,
            sampler: SamplerDesc::default(),
            mipmaps: false,
        };

        self.get_or_create(desc)
    }

    pub fn load_cpu_textures(&mut self) {
        for (id, tex) in self.storage.iter_mut() {
            if tex.state != TextureState::MetaOnly {
                continue;
            }

            match load_and_decode(&tex.desc) {
                Ok(cpu) => {
                    tex.state = TextureState::CpuReady(TextureInfo {
                        width: cpu.width,
                        height: cpu.height,
                        format: cpu.format,
                        byte_size: cpu.pixels.len(),
                    });

                    self.dirty_textures.push((id, UploadPayload::Ready(cpu)));
                    
                    info!("Loaded Texture {:?} {:?}", id, tex.state);
                }
                Err(e) => {
                    tex.state = TextureState::Fallback;
                    self.dirty_textures.push((id, UploadPayload::Fallback));

                    warn!("{:?}",e);
                    info!("Loaded dirty Texture {:?} {:?}", id, tex.state);
                }
            }

        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_texture_same_id() {
        let mut textures = TextureAssets::new();

        let key = TextureKey::File {
            path: "albedo.png".into(),
            color_space: ColorSpace::Rgba8,
            usage: TextureUsage::Albedo,
        };

        let desc = TextureDesc::File {
            key,
            sampler: SamplerDesc::default(),
            mipmaps: true,
        };

        let a = textures.get_or_create(desc.clone());
        let b = textures.get_or_create(desc);
        assert_eq!(a, b);
    }

    #[test]
    fn should_contain_white_texture_id() {
        let texture_assets = TextureAssets::new();

        let white_id = TextureId::white(&texture_assets);

        assert!(texture_assets.get_desc(white_id).is_some())
    }

    #[test]
    fn should_not_remove_shared_from_asset() {
        let mut textures = TextureAssets::new();

        let key = TextureKey::File {
            path: "albedo.png".into(),
            color_space: ColorSpace::Rgba8,
            usage: TextureUsage::Albedo,
        };

        let desc = TextureDesc::File {
            key,
            sampler: SamplerDesc::default(),
            mipmaps: true,
        };

        let _ = textures.get_or_create(desc.clone());
        let id = textures.get_or_create(desc);

        textures.remove(id);
        assert!(textures.get_desc(id).is_some());

        // now will remove ..
        textures.remove(id);
        assert!(textures.get_desc(id).is_none());
    }
}

/*
use wgpu::TextureFormat;

use crate::assets::texture::{CubeTexture, Texture};

    pub fn create_cubemap<P: AsRef<Path>>(
        &mut self,
        path: [P; 6],
        format: TextureFormat,
    ) -> Arc<CubeTexture> {
        let buffer0 = Self::read_bytes(path[0].as_ref()).unwrap();
        let buffer1 = Self::read_bytes(path[1].as_ref()).unwrap();
        let buffer2 = Self::read_bytes(path[2].as_ref()).unwrap();
        let buffer3 = Self::read_bytes(path[3].as_ref()).unwrap();
        let buffer4 = Self::read_bytes(path[4].as_ref()).unwrap();
        let buffer5 = Self::read_bytes(path[5].as_ref()).unwrap();

        // Slice di slice
        let buffers: [&[u8]; 6] = [&buffer0, &buffer1, &buffer2, &buffer3, &buffer4, &buffer5];
        let cubemap = CubeTexture::new(&self.device, &self.queue, &buffers, format);

        Arc::new(cubemap)
    } */

/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils;
    const HDR_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/core/newport_loft.hdr");

    fn create_manager() -> TextureManager {
        let (device, queue) = test_utils::get_device_and_queue();
        TextureManager::new(device.clone(), queue.clone())
    }

    #[test]
    fn should_create_texture_manager() {
        let manager = create_manager();

        assert!(manager.textures.is_empty());
    }

    #[test]
    fn should_load_cube_texture() {
        let mut manager = create_manager();

        let images = [
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/test/right.png"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/test/left.png"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/test/top.png"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/test/bottom.png"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/test/front.png"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/test/back.png"),
        ];

        let cube = manager.create_cubemap(images, TextureFormat::Rgba8UnormSrgb);

        assert_eq!(cube.extent.depth_or_array_layers, 6);

        #[cfg(feature = "save_tests")]
        {
            let (device, queue) = test_utils::get_device_and_queue();
            test_utils::save_cubemap_cross(&device, &queue, "Skybox_result.png", &cube.inner)
                .unwrap();
        }
    }

    #[test]
    fn should_load_hdr_texture_rgba32float() {
        let mut manager = create_manager();

        let hdr = manager.create_texture(HDR_PATH, TextureFormat::Rgba32Float);

        assert_eq!(hdr.inner.format(), TextureFormat::Rgba32Float);
    }

    /// Hdr
    #[test]
    fn should_load_hdr_texture_rgba16float() {
        let mut manager = create_manager();

        let hdr = manager.create_texture(HDR_PATH, TextureFormat::Rgba16Float);

        assert_eq!(hdr.inner.format(), TextureFormat::Rgba16Float);
        assert!(hdr.inner.width() > 0);
        assert!(hdr.inner.height() > 0);
        assert_eq!(hdr.inner.mip_level_count(), 1); // <- no mipmaps
        assert_eq!(hdr.inner.depth_or_array_layers(), 1); // <- 2D texture
        assert_eq!(hdr.inner.dimension(), wgpu::TextureDimension::D2);

        #[cfg(feature = "save_tests")]
        {
            let (device, queue) = test_utils::get_device_and_queue();
            test_utils::save_texture(&device, &queue, "hdr.png", &hdr.inner, 0).unwrap();
        }
    }
}
 */
