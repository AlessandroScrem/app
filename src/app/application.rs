use std::path::PathBuf;

use crate::app::domain::events::DomainEvent;
use crate::assets::GlobalAssetId;
use crate::engine::RuntimeEvent;
use crate::gpu::GpuInternalCounters;
use crate::renderer::scene_renderer::FrameStats;
use crate::ui::{UiSnapshot, UiTextureResolver};
use crate::{Camera, Globals};
use legion::{Entity, World};

use crate::assets::asset_manager::AssetManager;

pub struct AppRenderData<'a> {
    pub asset_mgr: &'a AssetManager,
    pub world: &'a World,
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
    fn init(&mut self);
    fn render_data(&self) -> AppRenderData<'_>;
    fn on_update(&mut self, events: &mut Vec<RuntimeEvent>);
    fn on_resize(&mut self, width: u32, height: u32);
    fn on_drop(&mut self, path: PathBuf);
    fn on_close(&mut self);
    fn exit_requested(&self) -> bool;
    fn push_event(&mut self, event: DomainEvent);
    fn get_scene_snapshot<'a>(
        &'a self,
        texture_resolver: &'a dyn UiTextureResolver,
        frame_stats: FrameStats,
        gpu_counters: GpuInternalCounters,
        hdr_id: &'a Vec<GlobalAssetId>,
    ) -> UiSnapshot<'a>;
}
