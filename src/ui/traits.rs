use crate::assets::TextureId;
use crate::gpu::GpuInternalCounters;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UiTextureId(usize);

impl UiTextureId {
    pub const fn new(id: usize) -> Self {
        Self(id)
    }

    pub const fn raw(self) -> usize {
        self.0
    }
}

#[derive(Clone, Default)]
pub struct UiTextureRegistry {
    assets: HashMap<TextureId, UiTextureId>,
    shadow_map: Option<UiTextureId>,
}

impl UiTextureRegistry {
    pub fn asset(&self, id: TextureId) -> Option<UiTextureId> {
        self.assets.get(&id).copied()
    }

    pub fn shadow_map(&self) -> Option<UiTextureId> {
        self.shadow_map
    }

    pub(crate) fn set_asset(&mut self, asset: TextureId, texture: UiTextureId) {
        self.assets.insert(asset, texture);
    }

    pub(crate) fn remove_asset(&mut self, asset: &TextureId) {
        self.assets.remove(asset);
    }

    pub(crate) fn set_shadow_map(&mut self, texture: Option<UiTextureId>) {
        self.shadow_map = texture;
    }
}

#[allow(dead_code)]
pub trait InternalCounter {
    fn internal_counter(&self) -> GpuInternalCounters;
}
