use crate::gpu::pipeline_manager::PipelineManager;
use crate::prelude::*;

use crate::input::Input;
use std::sync::Arc;
use winit::window::WindowAttributes;
use winit::{dpi::PhysicalSize, event_loop::ActiveEventLoop};

use super::RunningApp;
use super::winit_bridge::CenterWindow;
use crate::app::{Application, HasAssetMgr};
use crate::gpu::{
    GpuCache, GpuContext, GpuManager, GpuMaterialCache, GpuMeshCache, GpuSurface, GpuTextureCache,
};
use crate::renderer::ImguiRender;

#[derive(Default)]
pub struct Engine<A: Application> {
    pub app: A,
    pub runtime: Option<RunningApp>,
}

impl<A: Application + HasAssetMgr> Engine<A> {
    pub fn resume(&mut self, event_loop: &ActiveEventLoop, size: PhysicalSize<u32>) {
        if self.runtime.is_some() {
            return;
        };

        debug!("App resumed");

        let attrs = WindowAttributes::default()
            .with_inner_size(size)
            .with_title("App");

        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("Failed to create window")
                .try_fit_center_to_monitor(),
        );

        self.app.init();

        // gpu resources
        let mut imgui_context = imgui::Context::create();
        let gpu_context = GpuContext::default();
        let gpu_surface = GpuSurface::new(
            gpu_context.adapter(),
            gpu_context.instance(),
            window.clone(),
        );
        let imgui_render = ImguiRender::new(
            &gpu_context.device,
            &gpu_context.queue,
            &window,
            &mut imgui_context,
            gpu_surface.get_config().format,
        );
        //

        let asset_mgr = self.app.asset_mgr_mut();

        // gpu resources
        let mut texture_cache = GpuTextureCache::new(&gpu_context.device, &gpu_context.queue);
        texture_cache.upload_textures(
            &mut asset_mgr.textures,
            &gpu_context.device,
            &gpu_context.queue,
        );
        let gpu_cache = GpuCache {
            textures: texture_cache,
            material: GpuMaterialCache::default(),
            mesh: GpuMeshCache::default(),
        };

        let gpu_manager = GpuManager::new(
            &gpu_context.device,
            &gpu_context.queue,
            gpu_surface.get_config().width,
            gpu_surface.get_config().height,
            &gpu_cache.textures,
            asset_mgr.skybox.get_id(),
        );

        let pipeline_manager = PipelineManager::new(
            &gpu_context.device,
            &gpu_manager,
            gpu_surface.get_config().format,
        );
        //

        let scene_renderer = SceneRenderer::new(&gpu_context);
        let uilayer = UiLayer::new(&window, imgui_context, gpu_context.get_adapter_string());

        self.runtime = Some(RunningApp {
            window: window.clone(),
            input: Input::new(),
            scene_renderer,
            imgui_render,
            uilayer,
            timer: Timer::new(),
            events: Vec::new(),
            gpu_context,
            gpu_surface,
            gpu_cache,
            gpu_manager,
            pipeline_manager,
        });

        window.request_redraw();
    }
}
