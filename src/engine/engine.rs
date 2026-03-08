use crate::prelude::*;

use crate::input::Input;
use std::sync::Arc;
use winit::window::WindowAttributes;
use winit::{dpi::PhysicalSize, event_loop::ActiveEventLoop};

use super::RunningApp;
use super::winit_bridge::CenterWindow;
use crate::app::{Application, HasAssetMgr};
use crate::gpu::{GpuContext, GpuSurface, ImguiRender};

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

        let timer = std::time::Instant::now();
        debug!("App resumed after  {} ms", timer.elapsed().as_millis());

        let window = {
            let wnd = event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_inner_size(size)
                        .with_title("App"),
                )
                .map(|w| w.try_fit_center_to_monitor())
                .expect("Failed to crate window");

            Arc::new(wnd)
        };

        self.app.init();
        debug!("App initialized in {} ms", timer.elapsed().as_millis());

        let mut imgui_context = imgui::Context::create();
        let asset_mgr = self.app.asset_mgr_mut();

        // gpu resources
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

        let renderer = Renderer::new(&gpu_context, &gpu_surface, asset_mgr);

        let adapter_string = gpu_context.get_adapter_string();
        let uilayer = UiLayer::new(&window, imgui_context, adapter_string);

        debug!("Renderer initialized in {} ms", timer.elapsed().as_millis());

        self.runtime = Some(RunningApp {
            window: window.clone(),
            input: Input::new(),
            renderer,
            uilayer,
            is_minimized: false,
            timer: Timer::new(),
            events: Vec::new(),
            gpu_context,
            gpu_surface,
            imgui_render
        });

        window.request_redraw();
    }
}
