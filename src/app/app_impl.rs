use super::{App, Application};
use crate::engine::RunningApp;

use crate::prelude::*;

pub trait HasAssetMgr {
    fn asset_mgr_mut(&mut self) -> &mut AssetManager;
}

impl HasAssetMgr for App {
    fn asset_mgr_mut(&mut self) -> &mut AssetManager {
        &mut self.asset_mgr
    }
}

impl Application for App {
    fn init(&mut self) {
        let timer = std::time::Instant::now();

        self.domain_events
            .queue
            .push_back(DomainEvent::Assets(AssetEvent::LoadGltf(
                "./assets/Lantern/Lantern.gltf".into(),
            )));
        let hdrpath = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/core/newport_loft.hdr");
        let hdr_id = self
            .asset_mgr
            .textures
            .from_file(hdrpath, renderer::TextureUsage::HDR16);
        self.asset_mgr.skybox = assets::asset_manager::SkyboxHandle::new(hdr_id);

        crate::entities::light::create(&mut self.current_scene.world, &self.resources);

        self.current_scene.schedule = crate::systems::create_current_scene_schedule_builder();

        debug!("App loader took {} ms", timer.elapsed().as_millis());
    }
    fn update(&mut self, runtime: &mut RunningApp) {

        self.update_domain_event();

        
        // load texture from file to cpu data
        self.asset_mgr.textures.load_cpu_textures();

        // upload texture from cpu data to gpu 
        runtime.renderer.upload_textures(&mut self.asset_mgr.textures);


        // Esegue `callback` ogni secondo , in base al clock interno.
        runtime
            .timer
            .trigger_every(std::time::Duration::from_secs(1), || {
                runtime.renderer.sync_imgui_texture();
                debug!("Sync_with_registry: ");
            });

        self.update_camera(&runtime.input);
        self.update_selected(runtime);
        self.update_scene();
        self.update_uilayer(runtime);
    }

    fn on_resize(&mut self, width: u32, height: u32) {
        let aspect = width.max(1) as f32 / height.max(1) as f32;
        self.camera.set_aspect(aspect);
    }
    fn render(&mut self, runtime: &mut RunningApp) {
        runtime.renderer.render(
            &self.asset_mgr,
            &self.current_scene.world,
            &mut self.resources,
            &self.camera,
            &self.globals,
            self.selected,
            &runtime.input,
            runtime.uilayer.get_draw_data(),
        );
    }
    fn on_close(&mut self) {
        info!("The close button was pressed; App stopping");
    }
}

