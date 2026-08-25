use super::{App, Application, HasAssetMgr};
use crate::app::Settings;
use crate::app::application::AppRenderData;
use crate::app::domain::events::SelectionEvent::SelectIbl;
use crate::app::domain::events::{AssetEvent, DomainEvent, SceneEvent};
use crate::assets::asset_manager::AssetManager;
use crate::assets::ibl_asset::IblAsset;
use crate::assets::{IblId, MaterialAsset, MeshAsset, TextureAsset, TextureId};
use crate::ecs::components::light;
use crate::engine::engine::EventBus;
use crate::gpu::GpuInternalCounters;
use crate::renderer::scene_renderer::FrameStats;
use crate::ui::{RenderStats, UiSnapshot, UiTextureResolver};

use crate::prelude::*;

impl HasAssetMgr for App {
    fn asset_mgr_mut(&mut self) -> &mut AssetManager {
        &mut self.asset_mgr
    }
}

impl Application for App {
    fn init(&mut self, bus: &mut EventBus) {
        let timer = std::time::Instant::now();

        self.settings = Settings::load();

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
        bus.send_domain(DomainEvent::Selection(SelectIbl(ibl_id)));
        //*****************************

        debug!("App initialized in {} ms", timer.elapsed().as_millis());
    }

    fn get_scene_snapshot<'a>(
        &'a self,
        texture_resolver: &'a dyn UiTextureResolver,
        frame_stats: FrameStats,
        gpu_counters: GpuInternalCounters,
        hdr_vec: &'a Vec<(TextureId, IblId)>,
    ) -> UiSnapshot<'a> {
        let root_snapshot = self.current_scene.get_root_snapshot();
        let comp_state = self
            .current_scene
            .get_selected_componet_state(&self.selected, &self.asset_mgr);

        let render_stats = RenderStats {
            gpu_counters,
            frame_stats,
            texture: self.asset_mgr.get_stats::<TextureAsset>(),
            mesh: self.asset_mgr.get_stats::<MeshAsset>(),
            material: self.asset_mgr.get_stats::<MaterialAsset>(),
        };

        UiSnapshot {
            texture_resolver,
            render_stats,
            camera: &self.camera,
            globals: &self.globals,
            root_snapshot,
            comp_state,
            selected: &self.selected,
            hovered: self.hovered,
            debug_texture_id: self.debug_texture_id,
            hdr_vec,
            selected_ibl: self.selected_ibl,
            scene_name: self.current_scene.filename.clone(),
            settings: self.settings.clone(),
        }
    }

    fn on_update(&mut self, bus: &mut EventBus) {
        self.update_domain_event(bus);
        self.current_scene.update_scene(bus, &self.globals);
    }

    fn on_resize(&mut self, width: u32, height: u32) {
        let aspect = width.max(1) as f32 / height.max(1) as f32;
        self.camera.set_aspect(aspect);
    }

    fn on_drop(&mut self, path: std::path::PathBuf, bus: &mut EventBus) {
        let ext = path.extension().unwrap_or_default().to_str();
        match ext {
            Some("json") => bus.send_domain(DomainEvent::Scene(SceneEvent::Open(path))),
            Some("gltf") => bus.send_domain(DomainEvent::Assets(AssetEvent::LoadGltf(path))),
            _ => {}
        }
    }

    fn render_data(&self) -> AppRenderData<'_> {
        AppRenderData {
            render_objects: &self.current_scene.render_objects,
            asset_mgr: &self.asset_mgr,
            camera: &self.camera,
            globals: &self.globals,
            selected: &self.selected,
        }
    }

    fn on_close(&mut self) {
        let _ = self.settings.save();
        info!("Exit requested; App stopping");
    }
}
