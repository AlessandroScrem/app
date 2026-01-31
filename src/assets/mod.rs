use std::{collections::HashMap, path::PathBuf};

use slotmap::SlotMap;
use slotmap::new_key_type;
use slotmap::secondary::Iter;

use crate::BoundingBox;
use crate::assets::vertexdata::MeshVertexData;
use crate::material_manager::MATERIAL_TEXTURE_COUNT;
use crate::material_manager::MaterialTextureSlot;
use crate::math::*;

pub mod asset_manager;
pub mod file;
pub mod gltf_loader;
pub mod material_manager;
pub mod mesh;
pub mod texture;
pub mod texture_manager;
pub mod vertexdata;

new_key_type! {
    pub struct TextureId;
    pub struct MaterialId;
    pub struct MeshId;
}

impl TextureId {
    pub fn white(assets: &TextureAssets) -> TextureId {
        assets.white()
    }
}

// Textures
#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum ColorSpace {
    Rgbaf32,
    Rgbaf16,
    Srgba8,
    Rgba8,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum TextureUsage {
    Albedo,
    Normal,
    MetallicRoughness,
    Emissive,
    Occlusion,
    HDR16,
    HDR32,
}

impl From<material_manager::MaterialTextureSlot> for TextureUsage {
    fn from(slot: material_manager::MaterialTextureSlot) -> Self {
        match slot {
            material_manager::MaterialTextureSlot::BaseColor => TextureUsage::Albedo,
            material_manager::MaterialTextureSlot::Normal => TextureUsage::Normal,
            material_manager::MaterialTextureSlot::MetallicRoughness => {
                TextureUsage::MetallicRoughness
            }
            material_manager::MaterialTextureSlot::Emissive => TextureUsage::Emissive,
            material_manager::MaterialTextureSlot::Occlusion => TextureUsage::Occlusion,
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

#[derive(Default, Clone)]
pub enum SamplerDesc {
    #[default]
    Linear,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum TextureKey {
    File {
        path: PathBuf,
        color_space: ColorSpace,
        usage: TextureUsage,
    },
    White,
}

#[derive(Clone)]
pub enum TextureDesc {
    File {
        key: TextureKey,
        sampler: SamplerDesc,
        mipmaps: bool,
    },
    White,
}

pub struct TextureAssets {
    pub storage: SlotMap<TextureId, TextureDesc>,
    lookup: HashMap<TextureKey, TextureId>,
    white: TextureId,
}

impl Default for TextureAssets {
    fn default() -> Self {
        let mut storage = SlotMap::with_key();
        let mut lookup = HashMap::new();
        let white_key = TextureKey::White;
        let white_id = storage.insert(TextureDesc::White);

        lookup.insert(white_key, white_id);

        Self {
            storage,
            lookup,
            white: white_id,
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

    pub fn get(&self, id: TextureId) -> Option<&TextureDesc> {
        self.storage.get(id)
    }

    pub fn get_or_create(&mut self, desc: TextureDesc) -> TextureId {
        match desc {
            TextureDesc::White => self.white,
            TextureDesc::File {
                key,
                sampler,
                mipmaps,
            } => {
                if let Some(id) = self.lookup.get(&key) {
                    return *id;
                }
                let id = self.storage.insert(TextureDesc::File {
                    key: key.clone(),
                    sampler,
                    mipmaps,
                });
                self.lookup.insert(key, id);
                id
            }
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

    pub fn iter(&self) -> impl Iterator<Item = (TextureId, &TextureDesc)> {
        self.storage.iter()
    }
}

// Materials
#[derive(Default, Hash, Eq, PartialEq, Clone)]
pub enum ShaderId {
    #[default]
    Pbr,
}

#[derive(Default, Hash, Eq, PartialEq, Clone)]
pub struct MaterialKey {
    pub shader: ShaderId,
    pub textures: [Option<TextureId>; MATERIAL_TEXTURE_COUNT],
}

#[derive(Clone)]
pub struct MaterialDesc {
    pub key: MaterialKey,
    use_texture_slot: [bool; MATERIAL_TEXTURE_COUNT],

    pub base_color_factor: Vec4,
    pub emissive_factor: Vec4,
    pub roughness_factor: f32,
    pub metallic_factor: f32,
    pub normal_scale: f32,
    pub occlusion_strength: f32,
}

impl MaterialDesc {
    pub fn get_texture_slot(&self, slot: MaterialTextureSlot) -> Option<TextureId> {
        self.key.textures.get(slot as usize).copied().flatten()
    }
}

#[derive(Default)]
pub struct MaterialAssets {
    storage: SlotMap<MaterialId, MaterialDesc>,
    lookup: HashMap<MaterialKey, MaterialId>,
}

impl MaterialAssets {
    pub fn get_or_create(
        &mut self,
        key: MaterialKey,
        desc_fn: impl FnOnce() -> MaterialDesc,
    ) -> MaterialId {
        if let Some(id) = self.lookup.get(&key) {
            return *id;
        }

        let desc = desc_fn();
        let id = self.storage.insert(desc);
        self.lookup.insert(key, id);
        id
    }

    pub fn get(&self, id: MaterialId) -> Option<&MaterialDesc> {
        self.storage.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (MaterialId, &MaterialDesc)> {
        self.storage.iter()
    }
}

// Meshes
pub struct MeshDesc {
    pub vertices: Vec<MeshVertexData>,
    pub indices: Vec<u32>,
    pub submeshes: Vec<SubMesh>,
    pub bounds: BoundingBox,
}

pub struct SubMesh {
    pub index_range: std::ops::Range<u32>,
    pub base_vertex: u32,
    pub material: MaterialId,
}

#[derive(Hash, Eq, PartialEq)]
pub enum MeshSource {
    File {
        path: PathBuf,
        index: usize, // submesh index nel file
    },
    Generated {
        shape: Primitive,
        params: [u32; 4],
    },
}

#[derive(Hash, Eq, PartialEq)]
pub enum Primitive {
    Cube,
    Quad,
    Sphere,
    Cylinder,
    Grid,
}

#[derive(Hash, Eq, PartialEq)]
pub struct MeshKey {
    pub source: MeshSource,
}

#[derive(Default)]
pub struct MeshAssets {
    storage: SlotMap<MeshId, MeshDesc>,
    lookup: HashMap<MeshKey, MeshId>,
}

impl MeshAssets {
    pub fn get_or_create(&mut self, key: MeshKey, desc_fn: impl FnOnce() -> MeshDesc) -> MeshId {
        if let Some(id) = self.lookup.get(&key) {
            return *id;
        }

        let desc = desc_fn();
        let id = self.storage.insert(desc);
        self.lookup.insert(key, id);
        id
    }

    pub fn get(&self, id: MeshId) -> Option<&MeshDesc> {
        self.storage.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (MeshId, &MeshDesc)> {
        self.storage.iter()
    }
}

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

    assert!(texture_assets.get(white_id).is_some())
}
