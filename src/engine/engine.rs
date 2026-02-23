use std::sync::Arc;

use winit::window::WindowAttributes;
use winit::{dpi::PhysicalSize, event_loop::ActiveEventLoop};
use crate::input::Input;
use crate::prelude::*;

use crate::app::{Application, HasAssetMgr};
use crate::timer::Timer;
use super::{RunningApp};
use super::winit_bridge::CenterWindow;

#[derive(Default)]
pub(crate) struct Engine<A: Application> {
    pub(crate) app: A,
    pub(crate) runtime: Option<RunningApp>,
}

impl <A: Application + HasAssetMgr> Engine<A> {
    pub(crate) fn resume(&mut self, event_loop: &ActiveEventLoop, size: PhysicalSize<u32> ) {
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

        let mut context = imgui::Context::create();
        let asset_mgr = self.app.asset_mgr_mut();
        let renderer = Renderer::new(window.clone(), &mut context, asset_mgr);
        let adapter_string = renderer.get_adapter_string();
        let uilayer = UiLayer::new(&window, context, adapter_string);

        debug!("Renderer initialized in {} ms", timer.elapsed().as_millis());

        self.runtime = Some(RunningApp {
            window: window.clone(),
            input: Input::new(),
            renderer,
            uilayer,
            is_minimized: false,
            timer: Timer::new(),
            events: Vec::new(),
        });

        window.request_redraw();

    }
}