pub(crate) mod bindgroup;
pub(crate) mod bindgroup_layout;
pub(crate) mod buffer;
pub(crate) mod framebuffer;
pub(crate) mod internalcounter;
pub(crate) mod material;
pub(crate) mod mesh;
pub(crate) mod texture;

use std::collections::HashSet;

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
pub(crate) use super::context::*;
pub(crate) use super::manager::*;

pub struct GpuCache {
    pub mesh: GpuMeshCache,
    pub material: GpuMaterialCache,
    pub textures: GpuTextureCache,
}

pub struct SyncInput<'a, Id, T> {
    pub id: Id,
    pub data: &'a T,
}

/* impl GpuCache {
    pub fn sync_caches(
        &mut self,
        gpu_context: &GpuContext,
        gpu_manager: &GpuManager,
        asset_mgr: &AssetManager,
    ) {

        let mesh_input: Vec<SyncInput<MeshId, MeshDesc>> = asset_mgr
            .meshes
            .iter()
            .map(|a| SyncInput {
                id: a.0,
                data: &a.1.desc,
            })
            .collect();

        let material_input: Vec<SyncInput<MaterialId, MaterialDesc>> = asset_mgr
            .materials
            .iter()
            .map(|a| SyncInput { id: a.0, data: a.1 })
            .collect();

        let texture_input: Vec<SyncInput<TextureId, Option<TextureDesc>>> = asset_mgr
            .textures
            .iter()
            .map(|a| SyncInput { id: a.0, data: &a.1.desc })
            .collect();

        // Sync Meshes
        self.mesh.sync(&gpu_context.device, &mesh_input);

        // Sync Meshes
        self.material.sync(
            &mut self.textures,
            &gpu_context.device,
            gpu_manager,
            &material_input,
        );

        // Sync Textures: are already on sync after upload, or fallback
        self.textures.sync(&texture_input);

    }
} */
