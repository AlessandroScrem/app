use std::{
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use crate::ResourceId;

#[derive(Debug)]
pub struct AssetHandle<T> {
    id: ResourceId,
    marker: PhantomData<T>,
}

impl<T> AssetHandle<T> {
    pub fn new(id: ResourceId) -> Self {
        Self {
            id,
            marker: PhantomData,
        }
    }

    pub fn id(&self) -> ResourceId {
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
