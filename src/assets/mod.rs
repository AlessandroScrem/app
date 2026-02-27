use std::{collections::HashMap, path::PathBuf};

use slotmap::SlotMap;
use slotmap::new_key_type;

pub(crate) mod asset_manager;
pub(crate) mod file;
pub(crate) mod gltf_loader;
pub(crate) mod material_asset;
pub(crate) mod texture_asset;
pub(crate) mod mesh_asset;
pub(crate) mod vertexdata;
pub(crate) mod texture_upload;
pub(crate) mod image_decoder;

new_key_type! {
    pub(crate) struct TextureId;
    pub(crate) struct MaterialId;
    pub(crate) struct MeshId;
}

pub(crate) use crate::assets::vertexdata::MeshVertexData;
pub(crate) use texture_asset::*;
pub(crate) use mesh_asset::*;
pub(crate) use material_asset::*;
pub(crate) use crate::prelude::*;

#[test]
fn sync_add_remove_reuse() {
    use slotmap::{SlotMap, SecondaryMap, new_key_type};

    new_key_type! { pub(crate) struct MeshId; }

    // Asset storage (AssetManager side)
    let mut storage: SlotMap<MeshId, u32> = SlotMap::with_key();

    // GPU registry (Renderer side)
    let mut gpu_registry: SecondaryMap<MeshId, &'static str> = SecondaryMap::new();

    // --- FRAME 1: ADD ---
    let mesh = storage.insert(42);

    // Sync
    for (id, value) in storage.iter() {
        if !gpu_registry.contains_key(id) {
            gpu_registry.insert(id, "GPU_MESH");
            println!("Upload mesh {}", value);
        }
    }

    assert!(gpu_registry.contains_key(mesh));

    // --- FRAME 2: REMOVE ---
    storage.remove(mesh);

    // Sync cleanup
    gpu_registry.retain(|id, _| storage.contains_key(id));

    assert!(!gpu_registry.contains_key(mesh));
    assert!(!storage.contains_key(mesh));

    // --- FRAME 3: REUSE SLOT ---
    let reused = storage.insert(100);

    // Se SlotMap riusa lo slot:
    // reused potrebbe avere stesso index ma generation diversa.
    assert_ne!(mesh, reused); // 🔥 generational safety

    // Sync
    for (id, value) in storage.iter() {
        if !gpu_registry.contains_key(id) {
            gpu_registry.insert(id, "GPU_MESH_REUSED");
            println!("Upload mesh {}", value);
        }
    }

    assert!(gpu_registry.contains_key(reused));
    assert!(!gpu_registry.contains_key(mesh)); // vecchia key resta invalida
}

