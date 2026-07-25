use crate::prelude::*;

use std::sync::Arc;
use winit::window::WindowAttributes;
use winit::{dpi::PhysicalSize, event_loop::ActiveEventLoop};

use super::RunningApp;
use super::winit_bridge::CenterWindow;
use crate::app::{Application, HasAssetMgr};

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

        self.runtime = Some(RunningApp::new(window.clone()));

        window.request_redraw();
    }
}
