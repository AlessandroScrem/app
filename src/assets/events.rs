
use crate::assets::asset_id::AssetHandle;

#[derive(Debug, Clone, Copy)]
pub enum AssetEvent<T> {
    Created(AssetHandle<T>),
    Modified(AssetHandle<T>),
    Removed(AssetHandle<T>),
}