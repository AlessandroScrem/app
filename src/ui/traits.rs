use crate::assets::TextureId;
use crate::gpu::caches::internalcounter::GpuInternalCounters;
use imgui::TextureId as ImguiTextureId;

#[allow(dead_code)]
pub enum UiTexture {
    Engine(TextureId),
    Builtin(ImguiTextureId),
}

pub trait UiTextureResolver {
    fn resolve(&self, tex: UiTexture) -> Option<ImguiTextureId>;
}

pub trait InternalCounter {
    fn internal_counter(&self) -> GpuInternalCounters;
}
