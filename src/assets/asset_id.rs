use std::{
    any::TypeId,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use crate::assets::asset_storage::Asset;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalAssetId {
    pub type_id: TypeId,
    pub id: AssetId,
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
            id: value.id,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct AssetId {
    pub index: u32,
    pub generation: u32,
}

#[derive(Debug)]
pub struct AssetHandle<T> {
    id: AssetId,
    marker: PhantomData<T>,
}

impl<T> AssetHandle<T> {
    pub fn new(id: AssetId) -> Self {
        Self {
            id,
            marker: PhantomData,
        }
    }

    pub fn id(&self) -> AssetId {
        self.id
    }
}

impl<T> Copy for AssetHandle<T> {}

impl<T> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for AssetHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for AssetHandle<T> {}

impl<T> Hash for AssetHandle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
