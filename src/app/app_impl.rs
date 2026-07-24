use super::{App, Application, HasAssetMgr};
use crate::app::application::AppRenderData;
use crate::app::domain::events::{AssetEvent, DomainEvent};
use crate::assets::{GlobalAssetId, TextureAsset};
use crate::assets::asset_manager::AssetManager;
use crate::assets::ibl_asset::IblAsset;
use crate::ecs::components::light;
use crate::engine::RuntimeEvent;
use crate::gpu::GpuInternalCounters;
use crate::renderer::scene_renderer::FrameStats;
use crate::ui::{UiSnapshot, UiTextureResolver};

use crate::prelude::*;

impl HasAssetMgr for App {
    fn asset_mgr_mut(&mut self) -> &mut AssetManager {
        &mut self.asset_mgr
    }
}

impl Application for App {
    fn init(&mut self) {
        let timer = std::time::Instant::now();

        //*****************************
        // Create Light
        light::create(&mut self.current_scene.world);
        //*****************************

        //*****************************
        // Create Ibl
        const HDRPATH: &str = crate::asset_path!("core/Cannon_Exterior.hdr");
        let hdr_texture_asset =
            TextureAsset::from_file(HDRPATH, crate::assets::texture_asset::TextureUsage::HDR16);

        let hdr_id = self.asset_mgr.add::<TextureAsset>(hdr_texture_asset);
        let ibl_id = self
            .asset_mgr
            .add::<IblAsset>(IblAsset::new(hdr_id, HDRPATH));
        self.ibl_id = Some(ibl_id);
        //*****************************

        self.current_scene.schedule = crate::ecs::create_current_scene_schedule_builder();

        debug!("App initialized in {} ms", timer.elapsed().as_millis());
    }

    fn push_event(&mut self, event: DomainEvent) {
        self.push_event(event);
    }

    fn get_scene_snapshot<'a>(
        &'a self,
        texture_resolver: &'a dyn UiTextureResolver,
        frame_stats: FrameStats,
        gpu_counters: GpuInternalCounters,
        hdr_id: Option<GlobalAssetId>,
    ) -> UiSnapshot<'a> {
        self.get_scene_snapshot(texture_resolver, frame_stats, gpu_counters, hdr_id)
    }

    fn on_update(&mut self, events: &mut Vec<RuntimeEvent>) {
        self.update_domain_event();
        self.update_scene(events);
    }

    fn on_resize(&mut self, width: u32, height: u32) {
        let aspect = width.max(1) as f32 / height.max(1) as f32;
        self.camera.set_aspect(aspect);
    }

    fn on_drop(&mut self, path: std::path::PathBuf) {
        self.domain_events
            .queue
            .push_back(DomainEvent::Assets(AssetEvent::LoadGltf(path)));
    }

    fn render_data(&self) -> AppRenderData<'_> {
        AppRenderData {
            asset_mgr: &self.asset_mgr,
            world: &self.current_scene.world,
            camera: &self.camera,
            globals: &self.globals,
            selected: self.selected,
        }
    }

    fn on_close(&mut self) {
        self.exit_requested = true;
        info!("Exit requested; App stopping");
    }

    fn exit_requested(&self) -> bool {
        self.exit_requested
    }
}
