use crate::assets::TextureId;
use crate::gpu::GpuInternalCounters;
use crate::gpu::caches::FramebufferKind;
use imgui::TextureId as ImguiTextureId;

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum UiTexture {
    Engine(TextureId),
    Builtin(ImguiTextureId),
    Framebuffer(FramebufferKind),
    ShadowMap,
}

#[allow(dead_code)]
pub trait UiTextureResolver {
    fn resolve(&self, tex: UiTexture) -> Option<ImguiTextureId>;
}

#[allow(dead_code)]
pub trait InternalCounter {
    fn internal_counter(&self) -> GpuInternalCounters;
}
