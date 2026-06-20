use std::any::{Any, TypeId};
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

pub mod asset_id;
pub mod asset_storage;
mod dependency_graph;
pub mod resource_stats;

#[cfg(test)]
mod test_gam_api;
#[cfg(test)]
mod test_gam_load_api;

use asset_id::{AssetHandle, AssetId};
use asset_storage::{Asset, AssetStorage};
use dependency_graph::DependencyGraph;
use resource_stats::ResourceStats;

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

    fn remove_by_id(&mut self, id: AssetId)->usize;
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

    fn remove_by_id(&mut self, id: AssetId)-> usize {
        self.inner.remove_by_id(id)
    }
}

#[derive(Default)]
pub struct GlobalAssetManager {
    storages: HashMap<TypeId, Box<dyn ErasedStorage>>,
    stats: HashMap<TypeId, ResourceStats>,

    key_index: KeyRegistry,

    ref_count: HashMap<GlobalAssetId, u32>,

    graph: DependencyGraph,

    events: VecDeque<AssetEvent>,
}

impl GlobalAssetManager {
    pub fn new() -> GlobalAssetManager {
        GlobalAssetManager {
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

impl GlobalAssetManager {
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

impl GlobalAssetManager {
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

impl GlobalAssetManager {
    pub fn remove(&mut self, id: GlobalAssetId) {
        self.release(id);
    }

    fn remove_from_storage(&mut self, id: GlobalAssetId) {
        
        if let Some(storage) = self.storages.get_mut(&id.type_id) {
            let size = storage.remove_by_id(id.id);
            self.stats.get_mut(&id.type_id).map(|stat| stat.remove(size));
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

impl GlobalAssetManager {
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

impl GlobalAssetManager {
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

    impl asset_storage::Asset for Texture {
        type Key = String;

        fn key(&self) -> &Self::Key {
            &self.name
        }
    }

    #[test]
    fn add_dedup_and_refcount() {
        let mut mgr = GlobalAssetManager::new();

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
        let mut mgr = GlobalAssetManager::new();

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
        let mut mgr = GlobalAssetManager::new();

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
        let mut mgr = GlobalAssetManager::new();

        let tex = Texture { name: "tex".into() };

        let id = mgr.add(tex);

        mgr.remove(id);
        mgr.remove(id);
        mgr.remove(id);

        // nessun crash
        assert!(mgr.ref_count.get(&id).is_none());
    }
}
