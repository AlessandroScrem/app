pub(crate) mod mesh;
pub(crate) mod texture;
pub(crate) mod material;
pub(crate) mod internalcounter;
pub(crate) mod framebuffer;
pub(crate) mod bindgroup;
pub(crate) mod bindgroup_layout;
pub(crate) mod buffer;


pub (crate) use mesh::*;
pub (crate) use material::*;
pub (crate) use texture::*;
pub (crate) use internalcounter::*;
pub(crate) use framebuffer::*;
pub(crate) use bindgroup::*;
pub(crate) use bindgroup_layout::*;
pub(crate) use buffer::*;
pub(crate) use super::texture::{GpuTextureBuilder, GpuTextureUsage, GpuTexture, Dimension};
pub(crate) use super::static_textures;

pub(crate) use super::context::*;
pub(crate) use super::manager::*;
pub(crate) use super::assets::*;


pub struct GpuCache {
    pub mesh: GpuMeshCache,
    pub material: GpuMaterialCache,
    pub textures: GpuTextureCache,
}

impl GpuCache {
    pub fn sync_caches(
        &mut self,
        gpu_context: &GpuContext,
        gpu_manager: &GpuManager,
        asset_mgr: &AssetManager,
    ) {
        // Sync cleanup

        self.mesh.retain(&asset_mgr.meshes);
        self.material.retain(&asset_mgr.materials);
        self.textures.retain(&asset_mgr.textures);

        // Sync Textures: are already on sync after upload, or fallback

        // Sync Meshes
        for (id, _value) in asset_mgr.meshes.iter() {
            self.mesh
                .ensure(id, &asset_mgr.meshes, &gpu_manager, &gpu_context.device);
        }

        // Sync Materials (crate also textures)
        for (id, _value) in asset_mgr.materials.iter() {
            self.material.ensure(
                id,
                &mut self.textures,
                &asset_mgr,
                &gpu_manager,
                &gpu_context.device,
            );
        }
    }
}
