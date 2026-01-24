use std::{collections::HashMap, path::PathBuf};

use gltf::mesh::BoundingBox;
use slotmap::SlotMap;
use slotmap::new_key_type;

use crate::assets::vertexdata::MeshVertexData;
use crate::math::Vec3;

pub mod asset_manager;
pub mod material_manager;
pub mod mesh;
pub mod texture;
pub mod texture_manager;
pub mod vertexdata;
pub mod file;

new_key_type! {
    pub struct TextureId;
    pub struct MaterialId;
    pub struct MeshId;
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
    HDR16,
    HDR32,
}

impl TextureUsage {
    pub fn color_space(self) -> ColorSpace {
        match self {
            TextureUsage::Albedo => ColorSpace::Srgba8,
            TextureUsage::Emissive => ColorSpace::Srgba8,
            TextureUsage::Normal => ColorSpace::Rgba8,
            TextureUsage::MetallicRoughness => ColorSpace::Rgba8,
            TextureUsage::HDR16 => ColorSpace::Rgbaf16,
            TextureUsage::HDR32 => ColorSpace::Rgbaf32,
        }
    }
}

#[derive(Default, Clone)]
pub enum SamplerDesc {
    #[default]
    Linear,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct TextureKey {
    pub path: PathBuf,
    pub color_space: ColorSpace,
    pub usage: TextureUsage,
}

#[derive(Clone)]
pub struct TextureDesc {
    pub key: TextureKey,
    pub sampler: SamplerDesc,
    pub mipmaps: bool,
}

#[derive(Default)]
pub struct TextureAssets {
    pub storage: SlotMap<TextureId, TextureDesc>,
    lookup: HashMap<TextureKey, TextureId>,
}

impl TextureAssets {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_or_create(&mut self, desc: TextureDesc) -> TextureId {
        if let Some(id) = self.lookup.get(&desc.key) {
            return *id;
        }

        let id = self.storage.insert(desc.clone());
        self.lookup.insert(desc.key, id);
        id
    }

    pub fn from_file(&mut self, path: impl Into<PathBuf>, usage: TextureUsage) -> TextureId {
        let key = TextureKey {
            color_space: usage.color_space(),
            path: path.into(),
            usage,
        };

        let desc = TextureDesc {
            key,
            sampler: SamplerDesc::default(),
            mipmaps: false,
        };

        self.get_or_create(desc)
    }
}


// Materials
#[derive(Default, Hash, Eq, PartialEq, Clone)]
pub enum ShaderId {
    #[default]
    Pbr,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct MaterialKey {
    pub shader: ShaderId,
    pub textures: [Option<TextureId>; 6],
}

#[derive(Clone)]
pub struct MaterialDesc {
    pub key: MaterialKey,
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: Vec3,
}

pub struct MaterialAssets {
    storage: SlotMap<MaterialId, MaterialDesc>,
    lookup: HashMap<MaterialKey, MaterialId>,
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
    pub material: MaterialId,
}

#[derive(Hash, Eq, PartialEq)]
pub enum MeshSource {
    File {
        path: PathBuf,
        index: u32, // submesh index nel file
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

pub struct MeshAssets {
    storage: SlotMap<MeshId, MeshDesc>,
    lookup: HashMap<MeshKey, MeshId>,
}


#[test]
fn same_texture_same_id() {
    let mut textures = TextureAssets::new();

    let key = TextureKey {
        path: "albedo.png".into(),
        color_space: ColorSpace::Rgba8,
        usage: TextureUsage::Albedo,
    };

    let desc = TextureDesc {
        key,
        sampler: SamplerDesc::default(),
        mipmaps: true,
    };

    let a = textures.get_or_create(desc.clone());
    let b = textures.get_or_create(desc);
    assert_eq!(a, b);
}
