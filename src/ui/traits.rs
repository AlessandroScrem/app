use crate::ResourceId;
use crate::gpu::GpuInternalCounters;
use std::collections::HashMap;

#[derive(Clone, Default)]
pub struct UiTextures {
    textures: HashMap<ResourceId, imgui::TextureId>,
}

impl UiTextures {
    pub(crate) fn new(textures: HashMap<ResourceId, imgui::TextureId>) -> Self {
        Self { textures }
    }

    pub fn asset(&self, id: ResourceId) -> Option<imgui::TextureId> {
        self.textures.get(&id).copied()
    }
}

#[allow(dead_code)]
pub trait InternalCounter {
    fn internal_counter(&self) -> GpuInternalCounters;
}
