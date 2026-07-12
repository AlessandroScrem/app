use super::{App, Application, HandlesPicking, HasAssetMgr, HasUi};
use crate::app::application::AppRenderData;
use crate::app::domain::events::{AssetEvent, DomainEvent};
use crate::assets::TextureAsset;
use crate::assets::asset_manager::AssetManager;
use crate::assets::ibl_asset::IblAsset;
use crate::ecs::components::light;
use crate::input::Input;
use crate::ui::UiRuntimeContext;

use crate::prelude::*;

impl HasAssetMgr for App {
    fn asset_mgr_mut(&mut self) -> &mut AssetManager {
        &mut self.asset_mgr
    }
}

impl HandlesPicking for App {
    fn set_hovered(&mut self, hovered: Option<legion::Entity>) {
        self.hovered = hovered;
    }
}

impl HasUi for App {
    fn update_ui(&mut self, ctx: UiRuntimeContext<'_>) {
        self.update_uilayer(ctx);
    }
}

impl Application for App {
    fn init(&mut self) {
        let timer = std::time::Instant::now();

        //*****************************
        // Create Light
        light::create(&mut self.current_scene.world);
        // Turn Off All Lights
        // self.domain_events
        //     .queue
        //     .push_back(DomainEvent::Global(GlobalEvent::LightEnable(false)));

        //*****************************

        //*****************************
        // Create Ibl
        const HDRPATH: &str = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/core/Cannon_Exterior.hdr"
        );
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

    fn update(&mut self, input: &Input) {
        self.update_domain_event();

        self.update_camera(input);
        self.handle_selection_input(input);
        self.update_scene();
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
