use crate::assets::TextureId;
use crate::gpu::GpuInternalCounters;
use std::collections::HashMap;

#[derive(Clone, Default)]
pub struct UiTextureRegistry {
    assets: HashMap<TextureId, imgui::TextureId>,
    shadow_map: Option<imgui::TextureId>,
    material_preview: Option<imgui::TextureId>,
}

impl UiTextureRegistry {
    pub fn asset(&self, id: TextureId) -> Option<imgui::TextureId> {
        self.assets.get(&id).copied()
    }

    pub fn shadow_map(&self) -> Option<imgui::TextureId> {
        self.shadow_map
    }

    pub fn material_preview(&self) -> Option<imgui::TextureId> {
        self.material_preview
    }

    pub(crate) fn set_asset(&mut self, asset: TextureId, texture: imgui::TextureId) {
        self.assets.insert(asset, texture);
    }

    #[allow(dead_code)]
    pub(crate) fn remove_asset(&mut self, asset: &TextureId) {
        self.assets.remove(asset);
    }

    pub(crate) fn set_shadow_map(&mut self, texture: Option<imgui::TextureId>) {
        self.shadow_map = texture;
    }

    pub(crate) fn set_material_preview(&mut self, texture: Option<imgui::TextureId>) {
        self.material_preview = texture;
    }
}

#[allow(dead_code)]
pub trait InternalCounter {
    fn internal_counter(&self) -> GpuInternalCounters;
}
