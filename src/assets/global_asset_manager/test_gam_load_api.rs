

use super::*;

use crate::assets::material_asset::MaterialAsset;
use crate::assets::material_desc::MaterialDesc;
use crate::assets::mesh_asset::MeshAsset;
use crate::assets::texture_asset::TextureAsset;
use crate::assets::texture_asset::{SamplerDesc, TextureDesc, TextureUsage};
use crate::renderer::GpuTextureBuilder;
use crate::renderer::{GpuMaterial, GpuMesh, GpuTextureCache};

#[test]
fn same_texture_same_id() {
    let mut mgr = GlobalAssetManager::new();

    let desc = TextureDesc::File {
        path: "albedo.png".into(),
        usage: TextureUsage::Albedo,
        sampler: SamplerDesc::default(),
        mipmaps: true,
    };

    let texture = TextureAsset { desc: desc };

    let a = mgr.add(texture.clone());
    let b = mgr.add(texture);

    assert_eq!(a, b);
}

#[test]
fn texture_created_event() {
    use crate::assets::global_asset_manager::AssetEvent;
    use crate::assets::texture_upload::load_and_decode;
    use crate::test_utils;

    const TEXTURE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/core/white.png");

    let (device, queue) = test_utils::get_device_and_queue();

    let mut mgr = GlobalAssetManager::new();

    let desc = TextureDesc::File {
        path: TEXTURE_PATH.into(),
        usage: TextureUsage::Albedo,
        sampler: SamplerDesc::default(),
        mipmaps: true,
    };

    let texture = TextureAsset { desc: desc };

    let id = mgr.add(texture);

    assert_eq!(mgr.events.len(), 1);

    assert!(mgr.get::<TextureAsset>(id).is_some());

    let mut texture_cache = GpuTextureCache::new(device, queue);

    let events: Vec<AssetEvent> = mgr.events.drain(..).collect();

    for ev in events {
        let asset = mgr.get::<TextureAsset>(ev.id).unwrap();
        let data = load_and_decode(asset.desc.clone()).unwrap();
        let texture = GpuTextureBuilder::from_cpu(data).build(device, Some(queue));

        texture_cache.insert(ev.id, texture);
    }

    assert!(mgr.events.is_empty());
    assert!(texture_cache.contains_key(&id))
}

#[test]
fn material_created_event() {
    use crate::renderer::BindgroupLayoutKind;
    use crate::renderer::gpu::BindgroupLayoutCache;
    use crate::test_utils;

    let (device, queue) = test_utils::get_device_and_queue();
    let layout_cache = BindgroupLayoutCache::new(device);
    let bind_group_layout = layout_cache.get(BindgroupLayoutKind::Material);

    let texture_cache = GpuTextureCache::new(device, queue);

    let mut mgr = GlobalAssetManager::new();

    let material = MaterialAsset {
        desc: MaterialDesc::default(),
        key: "material".into(),
    };

    let id = mgr.add(material);

    assert_eq!(mgr.events.len(), 1);

    let grouped = mgr.drain_grouped_events();

    let mut gpu_materials: HashMap<GlobalAssetId, GpuMaterial> = Default::default();

    grouped.process_type::<MaterialAsset, _>(|_kind, events| {
        for ev in events {
            let asset = mgr.get::<MaterialAsset>(ev.id).unwrap();
            let gpu_material =
                GpuMaterial::new(&texture_cache, &asset.desc, device, bind_group_layout);

            gpu_materials.insert(ev.id, gpu_material);
        }
    });

    assert!(mgr.events.is_empty());

    assert_eq!(gpu_materials.len(), 1);
    assert!(gpu_materials.get(&id).is_some())
}

#[test]
fn mesh_created_event() {
    use crate::assets::mesh_asset::{MeshDesc, MeshSource};
    use crate::test_utils;

    let (device, _queue) = test_utils::get_device_and_queue();

    let mut mgr = GlobalAssetManager::new();

    const MESH_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/core/cube/cube.gltf");
    let mesh_source = MeshSource::File {
        path: MESH_PATH.into(),
        submesh_index: 0,
    };

    let mesh = MeshAsset {
        mesh_source,
        desc: MeshDesc::default(),
    };

    let id = mgr.add(mesh);

    assert_eq!(mgr.events.len(), 1);

    let grouped = mgr.drain_grouped_events();

    let mut gpu_meshes: HashMap<GlobalAssetId, GpuMesh> = Default::default();

    grouped.process_type::<MeshAsset, _>(|_kind, events| {
        for ev in events {
            let asset = mgr.get::<MeshAsset>(ev.id).unwrap();
            let gpu_mesh = GpuMesh::new(device, &asset.desc.vertices, &asset.desc.indices);
            gpu_meshes.insert(ev.id, gpu_mesh);
        }
    });

    assert!(mgr.events.is_empty());

    assert_eq!(gpu_meshes.len(), 1);
    assert!(gpu_meshes.get(&id).is_some())
}
