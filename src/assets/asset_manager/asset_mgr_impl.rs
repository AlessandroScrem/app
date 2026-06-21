use std::any::{Any, TypeId};
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

use super::asset_id::{AssetHandle, AssetId};
use super::asset_storage::{Asset, AssetStorage};
use super::dependency_graph::DependencyGraph;
use super::resource_stats::ResourceStats;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalAssetId {
    pub type_id: TypeId,
    pub id: AssetId,
}

impl Default for GlobalAssetId {
    fn default() -> Self {
        Self {
            type_id: TypeId::of::<()>(), // sentinella
            id: AssetId::default(),      // oppure INVALID
        }
    }
}

impl GlobalAssetId {
    pub fn new<T: Asset>(id: AssetId) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            id,
        }
    }
}

impl<T: 'static> From<&AssetHandle<T>> for GlobalAssetId {
    fn from(value: &AssetHandle<T>) -> Self {
        GlobalAssetId {
            type_id: TypeId::of::<T>(),
            id: value.id(),
        }
    }
}

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub enum AssetEventKind {
    Created,
    Updated,
    Removed,
    DependencyAdded,
    DependencyRemoved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssetEvent {
    pub id: GlobalAssetId,
    pub kind: AssetEventKind,
}

trait KeyMap {
    fn insert(&mut self, key: &dyn Any, id: GlobalAssetId);
    fn get(&self, key: &dyn Any) -> Option<GlobalAssetId>;
    fn remove(&mut self, id: GlobalAssetId);
}

struct TypedKeyMap<K: Eq + Hash + Clone> {
    map: HashMap<K, GlobalAssetId>,
}

impl<K: Eq + Hash + Clone + 'static> KeyMap for TypedKeyMap<K> {
    fn insert(&mut self, key: &dyn Any, id: GlobalAssetId) {
        let key = key.downcast_ref::<K>().unwrap();
        self.map.insert(key.clone(), id);
    }

    fn get(&self, key: &dyn Any) -> Option<GlobalAssetId> {
        let key = key.downcast_ref::<K>()?;
        self.map.get(key).copied()
    }

    fn remove(&mut self, id: GlobalAssetId) {
        self.map.retain(|_, v| *v != id);
    }
}

pub struct KeyRegistry {
    inner: HashMap<TypeId, Box<dyn KeyMap>>,
}
impl KeyRegistry {
    pub fn get<T: Asset>(&self, key: &T::Key) -> Option<GlobalAssetId> {
        let type_id = TypeId::of::<T>();

        self.inner.get(&type_id)?.get(key)
    }

    pub fn insert<T: Asset>(&mut self, key: T::Key, id: GlobalAssetId) {
        let type_id = TypeId::of::<T>();

        let entry = self.inner.entry(type_id).or_insert_with(|| {
            Box::new(TypedKeyMap::<T::Key> {
                map: HashMap::new(),
            })
        });

        entry.insert(&key, id);
    }

    pub fn remove(&mut self, id: GlobalAssetId) {
        for map in self.inner.values_mut() {
            map.remove(id);
        }
    }
}

impl Default for KeyRegistry {
    fn default() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }
}

trait ErasedStorage {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn remove_by_id(&mut self, id: AssetId) -> usize;
}

struct TypedStorage<T: Asset> {
    inner: AssetStorage<T>,
}

impl<T: Asset> ErasedStorage for TypedStorage<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn remove_by_id(&mut self, id: AssetId) -> usize {
        self.inner.remove_by_id(id)
    }
}

#[derive(Default)]
pub struct AssetManager {
    storages: HashMap<TypeId, Box<dyn ErasedStorage>>,
    stats: HashMap<TypeId, ResourceStats>,

    key_index: KeyRegistry,

    ref_count: HashMap<GlobalAssetId, u32>,

    graph: DependencyGraph,

    events: VecDeque<AssetEvent>,
}

impl AssetManager {
    pub fn new() -> AssetManager {
        AssetManager {
            storages: HashMap::new(),
            key_index: Default::default(),
            ref_count: HashMap::new(),
            graph: Default::default(),
            events: VecDeque::new(),
            stats: HashMap::new(),
        }
    }

    fn storage<T: Asset>(&self) -> &AssetStorage<T> {
        let id = TypeId::of::<T>();

        &self
            .storages
            .get(&id)
            .unwrap()
            .as_any()
            .downcast_ref::<TypedStorage<T>>()
            .unwrap()
            .inner
    }

    fn storage_mut<T: Asset>(&mut self) -> &mut AssetStorage<T> {
        let id = TypeId::of::<T>();

        self.storages.entry(id).or_insert_with(|| {
            Box::new(TypedStorage::<T> {
                inner: AssetStorage::<T>::default(),
            })
        });

        &mut self
            .storages
            .get_mut(&id)
            .unwrap()
            .as_any_mut()
            .downcast_mut::<TypedStorage<T>>()
            .unwrap()
            .inner
    }

    fn stats_mut<T: Asset>(&mut self) -> &mut ResourceStats {
        let id = TypeId::of::<T>();
        self.stats.entry(id).or_insert_with(ResourceStats::default)
    }
}

impl AssetManager {
    pub fn add<T: Asset>(&mut self, asset: T) -> GlobalAssetId {
        let key = asset.key().clone();

        let deps = asset.dependencies();

        if let Some(existing) = self.key_index.get::<T>(&key) {
            self.retain(existing);
            return existing;
        }

        // insert ResourceSize
        let size = asset.estimated_size();
        self.stats_mut::<T>().add(size);

        // create Assetid
        let handle = self.storage_mut::<T>().insert(asset);

        let gid = GlobalAssetId::new::<T>(handle.id());

        self.ref_count.insert(gid, 0);

        self.key_index.insert::<T>(key, gid);

        for dep in deps {
            self.graph.add(gid, dep);

            self.retain(dep);
        }

        self.events.push_back(AssetEvent {
            id: gid,
            kind: AssetEventKind::Created,
        });

        gid
    }

    pub fn get<T: Asset>(&self, id: GlobalAssetId) -> Option<&T> {
        if id.type_id != TypeId::of::<T>() {
            return None;
        }

        self.storage::<T>().get_by_id(id.id)
    }

    pub fn update<T: Asset>(&mut self, id: GlobalAssetId, asset: T) {
        if id.type_id != TypeId::of::<T>() {
            return;
        }

        if self.storage::<T>().get_by_id(id.id).is_none() {
            return;
        }

        let handle = AssetHandle::<T>::new(id.id);

        if let Some(existing) = self.storage_mut::<T>().get_mut(handle) {
            *existing = asset;

            self.events.push_back(AssetEvent {
                id,
                kind: AssetEventKind::Updated,
            });
        }
    }

    pub fn get_stats<T: Asset>(&self) -> ResourceStats {
        self.stats
            .get(&TypeId::of::<T>())
            .map_or_else(ResourceStats::default, Clone::clone)
    }
}

impl AssetManager {
    pub fn retain(&mut self, id: GlobalAssetId) {
        *self.ref_count.entry(id).or_insert(0) += 1;
    }

    pub fn release(&mut self, id: GlobalAssetId) {
        let should_destroy = match self.ref_count.get_mut(&id) {
            Some(count) => {
                *count = count.saturating_sub(1);
                *count == 0
            }
            None => false,
        };

        if should_destroy {
            self.remove_recursive(id);
        }
    }
}

impl AssetManager {
    pub fn remove(&mut self, id: GlobalAssetId) {
        self.release(id);
    }

    fn remove_from_storage(&mut self, id: GlobalAssetId) {
        if let Some(storage) = self.storages.get_mut(&id.type_id) {
            let size = storage.remove_by_id(id.id);
            self.stats
                .get_mut(&id.type_id)
                .map(|stat| stat.remove(size));
        }
    }

    fn remove_recursive(&mut self, id: GlobalAssetId) {
        // 1. prendi dipendenze PRIMA di rimuovere
        let dependencies = self.graph.dependencies_of(id);

        // 2. rimuovi dal grafo
        self.graph.remove_asset(id);

        // 3. rimuovi da storage
        self.remove_from_storage(id);

        // 4. rimuovi key + ref
        self.key_index.remove(id);
        self.ref_count.remove(&id);

        // 5. evento
        self.events.push_back(AssetEvent {
            id,
            kind: AssetEventKind::Removed,
        });

        // 6. rilascia dipendenze (IMPORTANTISSIMO ordine corretto)
        for dep in dependencies {
            self.release(dep);
        }
    }
}

impl AssetManager {
    pub fn drain_events(&mut self) -> Vec<AssetEvent> {
        self.events.drain(..).collect()
    }
}

#[derive(Default)]
pub struct GroupedEvents {
    inner: HashMap<(TypeId, AssetEventKind), Vec<AssetEvent>>,
}

impl GroupedEvents {
    pub fn type_groups(
        &self,
        type_id: TypeId,
    ) -> impl Iterator<Item = (AssetEventKind, &Vec<AssetEvent>)> {
        self.inner
            .iter()
            .filter_map(move |((tid, kind), events)| (*tid == type_id).then_some((*kind, events)))
    }
}

impl GroupedEvents {
    pub fn process_type<T: 'static, F>(&self, mut f: F)
    where
        F: FnMut(AssetEventKind, &Vec<AssetEvent>),
    {
        const ORDER: [AssetEventKind; 3] = [
            AssetEventKind::Created,
            AssetEventKind::Updated,
            AssetEventKind::Removed,
        ];
        let type_id = TypeId::of::<T>();

        for kind in ORDER {
            if let Some(events) = self.inner.get(&(type_id, kind)) {
                f(kind, events);
            }
        }
    }
}

impl AssetManager {
    pub fn drain_grouped_events(&mut self) -> GroupedEvents {
        let mut grouped: HashMap<(TypeId, AssetEventKind), Vec<AssetEvent>> = HashMap::new();

        for event in self.events.drain(..) {
            grouped
                .entry((event.id.type_id, event.kind))
                .or_default()
                .push(event);
        }

        GroupedEvents { inner: grouped }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct Texture {
        name: String,
    }

    impl Asset for Texture {
        type Key = String;

        fn key(&self) -> &Self::Key {
            &self.name
        }
    }

    #[test]
    fn add_dedup_and_refcount() {
        let mut mgr = AssetManager::new();

        let a = Texture { name: "tex".into() };

        let id1 = mgr.add(a.clone());
        assert_eq!(mgr.ref_count.get(&id1), Some(&0));

        // evento created
        let ev = mgr.events.back().unwrap();
        assert_eq!(ev.id, id1);
        assert_eq!(ev.kind, AssetEventKind::Created);

        // dedup
        let id2 = mgr.add(a.clone());
        assert_eq!(id1, id2);

        // refcount incrementato
        assert_eq!(mgr.ref_count.get(&id1), Some(&1));

        // 1 solo evento create
        assert_eq!(mgr.events.len(), 1);
    }

    #[test]
    fn remove_basic() {
        let mut mgr = AssetManager::new();

        let tex = Texture { name: "tex".into() };

        let id = mgr.add(tex.clone());

        mgr.remove(id);

        // deve essere rimosso
        assert!(mgr.ref_count.get(&id).is_none());

        // evento remove
        let ev = mgr.events.back().unwrap();
        assert_eq!(ev.id, id);
        assert_eq!(ev.kind, AssetEventKind::Removed);
    }

    #[test]
    fn retain_release() {
        let mut mgr = AssetManager::new();

        let tex = Texture { name: "tex".into() };

        let id = mgr.add(tex);

        mgr.retain(id);
        assert_eq!(mgr.ref_count.get(&id), Some(&1));

        mgr.release(id);
        // ora rimosso
        assert!(mgr.ref_count.get(&id).is_none());
    }

    #[test]
    fn remove_idempotent() {
        let mut mgr = AssetManager::new();

        let tex = Texture { name: "tex".into() };

        let id = mgr.add(tex);

        mgr.remove(id);
        mgr.remove(id);
        mgr.remove(id);

        // nessun crash
        assert!(mgr.ref_count.get(&id).is_none());
    }

    
    use crate::renderer::GpuTextureBuilder;
    use crate::renderer::{GpuMaterial, GpuMesh, GpuTextureCache};

    #[test]
    fn same_texture_same_id() {
        use crate::assets::texture_asset::*;
        let mut mgr = AssetManager::new();

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
        use crate::assets::texture_asset::*;
        use crate::assets::texture_upload::load_and_decode;
        use crate::test_utils;

        const TEXTURE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/core/white.png");

        let (device, queue) = test_utils::get_device_and_queue();

        let mut mgr = AssetManager::new();

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

        let events: Vec<super::AssetEvent> = mgr.events.drain(..).collect();

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
        use crate::assets::*;

        let (device, queue) = test_utils::get_device_and_queue();
        let layout_cache = BindgroupLayoutCache::new(device);
        let bind_group_layout = layout_cache.get(BindgroupLayoutKind::Material);

        let texture_cache = GpuTextureCache::new(device, queue);

        let mut mgr = AssetManager::new();

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
        use crate::assets::mesh_asset::*;
        use crate::test_utils;

        let (device, _queue) = test_utils::get_device_and_queue();

        let mut mgr = AssetManager::new();

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
}

#[cfg(test)]
mod test_api {
use super::*;
/// -----------------------------
/// MOCK ASSETS (USER SIDE)
/// -----------------------------

#[derive(Clone)]
struct Texture {
    name: String,
}


#[derive(Clone)]
struct Material {
    name: String,
    albedo: Option<GlobalAssetId>,
    normal: Option<GlobalAssetId>,
}


#[derive(Clone)]
struct Mesh {
    name: String,
    material: Option<GlobalAssetId>,
}

/// -----------------------------
/// IMPLEMENT Asset TRAIT
/// -----------------------------


impl Asset for Texture {
    type Key = String;
    
    fn key(&self) -> &Self::Key {
        &self.name
    }
}

impl Asset for Material {
    type Key = String;

    fn key(&self) -> &Self::Key {
        &self.name
    }

    fn dependencies(&self) -> Vec<GlobalAssetId> {
        let mut deps = Vec::new();

        if let Some(a) = self.albedo {
            deps.push(a);
        }

        if let Some(n) = self.normal {
            deps.push(n);
        }

        deps
    }
}

impl Asset for Mesh {
    type Key = String;

    fn key(&self) -> &Self::Key {
        &self.name
    }

    fn dependencies(&self) -> Vec<GlobalAssetId> {
        let mut deps = Vec::new();

        if let Some(m) = self.material {
            deps.push(m);
        }

        deps
    }
}

#[test]
fn mesh_removal_reduces_material_refcount() {
    let mut mgr = AssetManager::new();

    let mat = mgr.add(Material {
        name: "Mat".into(),
        albedo: None,
        normal: None,
    });

    let mesh1 = mgr.add(Mesh {
        name: "M1".into(),
        material: Some(mat),
    });

    let mesh2 = mgr.add(Mesh {
        name: "M2".into(),
        material: Some(mat),
    });

    assert_eq!(mgr.ref_count.get(&mat), Some(&2));

    mgr.remove(mesh1);
    // ✔ mesh1 tolto → materiale ancora vivo
    assert_eq!(mgr.ref_count.get(&mat), Some(&1));

    mgr.remove(mesh2);
    // ✔ mesh2 tolto → materiale ancora vivo
    assert!(mgr.ref_count.get(&mat).is_none());
}

#[test]
fn material_removal_reduces_texture_refcount() {
    let mut mgr = AssetManager::new();

    let tex = mgr.add(Texture {
        name: "T.png".into(),
    });

    let mat1 = mgr.add(Material {
        name: "M1".into(),
        albedo: Some(tex),
        normal: None,
    });

    let mat2 = mgr.add(Material {
        name: "M2".into(),
        albedo: Some(tex),
        normal: None,
    });

    assert_eq!(mgr.ref_count.get(&tex), Some(&2));

    mgr.remove(mat1);
    // texture ancora viva
    assert_eq!(mgr.ref_count.get(&tex), Some(&1));

    mgr.remove(mat2);
    // texture distrutta
    assert!(mgr.ref_count.get(&tex).is_none());
}

#[test]
fn retain_release_behavior() {
    let mut mgr = AssetManager::new();

    let tex = mgr.add(Texture {
        name: "Tex.png".into(),
    });

    // add crea già l'asset vivo (ref = 1 o 0 dipende da design, ma NON 0 stabile)
    assert!(mgr.ref_count.get(&tex).is_some());

    // primo retain
    mgr.retain(tex);
    assert_eq!(mgr.ref_count.get(&tex), Some(&1));

    // release finale → DEVE essere rimosso
    mgr.release(tex);
    assert!(mgr.ref_count.get(&tex).is_none());
}

#[test]
fn dedup_chain_mesh_material_texture() {
    let mut mgr = AssetManager::new();

    let tex = mgr.add(Texture {
        name: "T.png".into(),
    });

    let mat = mgr.add(Material {
        name: "M".into(),
        albedo: Some(tex),
        normal: None,
    });

    let mesh1 = mgr.add(Mesh {
        name: "A".into(),
        material: Some(mat),
    });

    let mesh2 = mgr.add(Mesh {
        name: "A".into(),
        material: Some(mat),
    });

    // mesh dedup
    assert_eq!(mesh1, mesh2);
}

}
