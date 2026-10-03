use crate::assets::TextureId;
use crate::ResourceId;
use crate::gpu::GpuInternalCounters;
use std::collections::HashMap;

#[derive(Clone, Default)]
pub struct UiTextureRegistry {
    textures: HashMap<ResourceId, imgui::TextureId>,
    shadow_map: Option<ResourceId>,
    material_preview: Option<ResourceId>,
}

impl UiTextureRegistry {
    pub fn texture(&self, id: ResourceId) -> Option<imgui::TextureId> {
        self.textures.get(&id).copied()
    }

    pub fn asset(&self, id: TextureId) -> Option<imgui::TextureId> {
        self.texture(id)
    }

    pub fn shadow_map(&self) -> Option<imgui::TextureId> {
        self.shadow_map.and_then(|id| self.texture(id))
    }

    pub fn material_preview(&self) -> Option<imgui::TextureId> {
        self.material_preview.and_then(|id| self.texture(id))
    }

    pub(crate) fn set_texture(&mut self, id: ResourceId, texture: imgui::TextureId) {
        self.textures.insert(id, texture);
    }

    pub(crate) fn set_shadow_map(&mut self, id: Option<ResourceId>) {
        self.shadow_map = id;
    }

    pub(crate) fn set_material_preview(&mut self, id: Option<ResourceId>) {
        self.material_preview = id;
    }
}

#[allow(dead_code)]
pub trait InternalCounter {
    fn internal_counter(&self) -> GpuInternalCounters;
}
