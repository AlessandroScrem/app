use crate::assets::texture_upload::{CpuTexture, TextureUploadSource, UploadPayload};

use super::*;
use std::cell::Cell;

impl TextureId {
    #[allow(dead_code)]
    fn white(assets: &TextureAssets) -> TextureId {
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
    #[allow(dead_code)]
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
}

#[derive(Clone, PartialEq, Debug)]
pub struct TextureInfo {
    pub width: u32,
    pub height: u32,
    pub format: ColorSpace,
    pub byte_size: usize,
}

impl From<&CpuTexture> for TextureInfo {
    fn from(value: &CpuTexture) -> Self {
        Self {
            width: value.width,
            height: value.height,
            format: value.format,
            byte_size: value.pixels.len(),
        }
    }
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
    pub desc: Option<TextureDesc>,
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

        
        let white_cpu = CpuTexture::white();

        let white_id = storage.insert(TextureAsset {
            state: TextureState::CpuReady(TextureInfo::from(&white_cpu)),
            desc: None,
            ref_count: Cell::new(1),
        });

        lookup.insert(TextureKey::White, white_id);

        Self {
            storage,
            lookup,
            white: white_id,
            dirty_textures: vec![(white_id, UploadPayload::Ready(white_cpu))],
        }
    }
}

impl TextureAssets {
    #[allow(dead_code)]
    pub fn new() -> Self {
        TextureAssets::default()
    }
    
    pub fn white(&self) -> TextureId {
        self.white
    }
    
    #[allow(dead_code)]
    pub fn get_desc(&self, id: TextureId) -> Option<&TextureDesc> {
        self.storage.get(id)?.desc.as_ref()
    }

    pub fn contains_key(&self, id: TextureId) -> bool {
        self.storage.contains_key(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (TextureId, &TextureAsset)> {
        self.storage.iter()
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
                // remove from storage
                if let Some(removed) = self.storage.remove(id) {
                    if let Some(TextureDesc::File { key, .. }) = removed.desc {
                        self.lookup.remove(&key);
                    }
                }
            }
        }
    }

    pub fn get_or_create(&mut self, desc: TextureDesc) -> TextureId {
        match desc {
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
                        desc: Some(TextureDesc::File {
                            key: key.clone(),
                            sampler,
                            mipmaps,
                        }),
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
        let results = super::texture_upload::load_cpu_textures_par(self.storage.iter());

        for (id, result) in results {
            self.storage
                .get_mut(id)
                .map(|asset| match result {
                    Ok(upload) => match upload {
                        UploadPayload::Ready(cpu) => {
                            asset.state = TextureState::CpuReady(TextureInfo::from(&cpu));
                            self.dirty_textures.push((id, UploadPayload::Ready(cpu)));
                            trace!("Asset Load Texture {:?} {:?}", id, asset.state);
                        }
                        UploadPayload::Fallback => {
                            asset.state = TextureState::Fallback;
                            self.dirty_textures.push((id, UploadPayload::Fallback));
                            trace!("Asset Load Fallback Texture {:?} {:?}", id, asset.state);
                        }
                    },
                    Err(e) => {
                        asset.state = TextureState::Fallback;
                        self.dirty_textures.push((id, UploadPayload::Fallback));
                        warn!("{:?} for id {:?} desc {:?}", e, id, asset.desc);
                    }
                })
                .unwrap_or_else(|| warn!("TextureId {:?} not found", id));
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

        assert!(texture_assets.contains_key(white_id))
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
