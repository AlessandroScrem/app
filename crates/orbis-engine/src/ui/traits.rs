use crate::ResourceId;
use crate::gpu::GpuInternalCounters;
use std::collections::HashMap;

#[derive(Clone, Default)]
pub struct UiTextures {
    textures: HashMap<ResourceId, imgui::TextureId>,
    pub shadow_map: Option<imgui::TextureId>,
    pub material_preview: Option<imgui::TextureId>,
}

impl UiTextures {
    pub(crate) fn new(
        textures: HashMap<ResourceId, imgui::TextureId>,
        shadow_map: Option<imgui::TextureId>,
        material_preview: Option<imgui::TextureId>,
    ) -> Self {
        Self {
            textures,
            shadow_map,
            material_preview,
        }
    }

    pub fn asset(&self, id: ResourceId) -> Option<imgui::TextureId> {
        self.textures.get(&id).copied()
    }
}

#[allow(dead_code)]
pub trait InternalCounter {
    fn internal_counter(&self) -> GpuInternalCounters;
}
