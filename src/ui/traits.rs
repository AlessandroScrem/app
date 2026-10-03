use crate::ResourceId;
use crate::gpu::GpuInternalCounters;
use std::collections::HashMap;

#[derive(Clone, Default)]
pub struct UiTextureRegistry {
    textures: HashMap<ResourceId, imgui::TextureId>,
}

impl UiTextureRegistry {
    pub fn texture(&self, id: ResourceId) -> Option<imgui::TextureId> {
        self.textures.get(&id).copied()
    }

    pub(crate) fn set_texture(&mut self, id: ResourceId, texture: imgui::TextureId) {
        self.textures.insert(id, texture);
    }
}

#[allow(dead_code)]
pub trait InternalCounter {
    fn internal_counter(&self) -> GpuInternalCounters;
}
