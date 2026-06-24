mod bindgroup;
mod bindgroup_layout;
mod buffer;
mod framebuffer;
mod internalcounter;
mod material;
mod mesh;
mod texture;


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

pub struct GpuCache {
    pub mesh: GpuMeshCache,
    pub material: GpuMaterialCache,
    pub textures: GpuTextureCache,
}

crate::impl_debug_drop!(GpuMaterial);
crate::impl_debug_drop!(GpuMesh);
// impl_debug_drop!(GpuTexture);
