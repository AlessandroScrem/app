use crate::assets::{IblId, TextureId};
use crate::engine::engine::EventBus;
use crate::gpu::GpuInternalCounters;
use crate::renderer::render_objects::RenderObjects;
use crate::renderer::scene_renderer::FrameStats;
use crate::ui::{UiSnapshot, UiTextureResolver};
use crate::{Camera, Globals};
use legion::Entity;
use std::path::PathBuf;

use crate::assets::asset_manager::AssetManager;

pub struct AppRenderData<'a> {
    pub render_objects: &'a RenderObjects,
    pub asset_mgr: &'a AssetManager,
    pub camera: &'a Camera,
    pub globals: &'a Globals,
    pub selected: Option<Entity>,
}

pub trait HasAssetMgr {
    fn asset_mgr_mut(&mut self) -> &mut AssetManager;
}

pub trait RuntimeApp: Application + HasAssetMgr {}

impl<T> RuntimeApp for T where T: Application + HasAssetMgr {}

pub trait Application {
    fn init(&mut self, bus: &mut EventBus);
    fn render_data(&self) -> AppRenderData<'_>;
    fn on_update(&mut self, bus: &mut EventBus);
    fn on_resize(&mut self, width: u32, height: u32);
    fn on_drop(&mut self, path: PathBuf, bus: &mut EventBus);
    fn on_close(&mut self);
    fn get_scene_snapshot<'a>(
        &'a self,
        texture_resolver: &'a dyn UiTextureResolver,
        frame_stats: FrameStats,
        gpu_counters: GpuInternalCounters,
        hdr_id: &'a Vec<(TextureId, IblId)>,
    ) -> UiSnapshot<'a>;
}
