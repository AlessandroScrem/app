use super::{App, Application};
use crate::engine::RunningApp;

use crate::gpu::{GpuCache, GpuContext, GpuManager, PipelineManager};
use crate::input::Input;
use crate::prelude::*;

pub struct RuntimeContext<'a> {
    pub gpu_context: &'a GpuContext,
    pub gpu_manager: &'a mut GpuManager,
    pub pipeline_manager: &'a PipelineManager,
    pub gpu_cache: &'a mut GpuCache,
    pub input: &'a mut Input,
}

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

        // const HDRPATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/core/newport_loft.hdr");
        const HDRPATH: &str = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/core/Cannon_Exterior.hdr"
        );
        let hdr_id = self
            .asset_mgr
            .textures
            .from_file(HDRPATH, renderer::TextureUsage::HDR16);

        self.asset_mgr.skybox = assets::asset_manager::SkyboxHandle::new(hdr_id);
        self.asset_mgr.textures.load_cpu_textures();

        crate::entities::light::create(&mut self.current_scene.world, &self.resources);
        self.current_scene.schedule = crate::systems::create_current_scene_schedule_builder();

        debug!("App initialized in {} ms", timer.elapsed().as_millis());
    }

    fn update(&mut self, runtime: &mut RunningApp) {
        self.update_domain_event();

        // load texture from file to cpu data
        self.asset_mgr.textures.load_cpu_textures();

        // upload texture from cpu data to gpu
        runtime.gpu_cache.textures.upload_textures(
            &mut self.asset_mgr.textures,
            &runtime.gpu_context.device,
            &runtime.gpu_context.queue,
        );

        // Esegue `callback` ogni secondo , in base al clock interno.
        runtime
            .timer
            .trigger_every(std::time::Duration::from_secs(1), || {
                runtime
                    .imgui_render
                    .sync_imgui_texture(&runtime.gpu_context, &mut runtime.gpu_cache);
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

    fn on_drop(&mut self, path: std::path::PathBuf) {
        self.domain_events.queue.push_back(DomainEvent::Assets(AssetEvent::LoadGltf(path)));
    }

    fn render(&mut self, runtime: &mut RunningApp) {
        let mut encoder = runtime.gpu_context.create_encoder();

        if let Some(frame) = runtime.gpu_surface.get_frame() {
            
            let target = frame.texture.create_view(&Default::default());
            let size: (u32, u32) = (
                runtime.gpu_surface.get_config().width,
                runtime.gpu_surface.get_config().height,
            );

            let RunningApp {
                scene_renderer,
                gpu_context,
                gpu_manager,
                pipeline_manager,
                gpu_cache,
                input,
                ..
            } = runtime;

            let mut context = RuntimeContext {
                gpu_context,
                gpu_manager,
                pipeline_manager,
                gpu_cache,
                input,
            };

            scene_renderer.render(
                &mut context,
                &mut encoder,
                &target,
                size,
                &self.asset_mgr,
                &self.current_scene.world,
                &self.camera,
                &self.globals,
                self.selected,
            );

            runtime.imgui_render.render(
                runtime.uilayer.get_draw_data(),
                &mut encoder,
                &target,
                &runtime.gpu_context.device,
                &runtime.gpu_context.queue,
            );

            runtime.gpu_context.queue.submit([encoder.finish()]);
            frame.present();
        }
    }

    fn on_close(&mut self) {
        info!("The close button was pressed; App stopping");
    }
}
