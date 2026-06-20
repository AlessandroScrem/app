pub(crate) mod bindgroup;
pub(crate) mod bindgroup_layout;
pub(crate) mod buffer;
pub(crate) mod framebuffer;
pub(crate) mod internalcounter;
pub(crate) mod material;
pub(crate) mod mesh;
pub(crate) mod texture;


pub(crate) use super::static_textures;
pub(crate) use super::texture::{Dimension, GpuTexture, GpuTextureBuilder, GpuTextureUsage};
pub(crate) use bindgroup::*;
pub(crate) use bindgroup_layout::*;
pub(crate) use buffer::*;
pub(crate) use framebuffer::*;
pub(crate) use internalcounter::*;
pub(crate) use material::*;
pub(crate) use mesh::*;
pub(crate) use texture::*;

pub(crate) use super::assets::*;
pub(crate) use super::manager::*;

pub struct GpuCache {
    pub mesh: GpuMeshCache,
    pub material: GpuMaterialCache,
    pub textures: GpuTextureCache,
}

