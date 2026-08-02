
use std::collections::VecDeque;
use std::sync::Arc;
use winit::window::WindowAttributes;
use winit::{dpi::PhysicalSize, event_loop::ActiveEventLoop};

use super::Runtime;
use super::winit_bridge::CenterWindow;
use crate::app::domain::events::DomainEvent;
use crate::app::{Application, HasAssetMgr};
use crate::engine::RuntimeEvent;
use crate::{prelude::*};

pub struct EventBus {
    domain: VecDeque<DomainEvent>,
    runtime: VecDeque<RuntimeEvent>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
impl EventBus {
    pub fn new() -> Self {
        Self {
            domain: VecDeque::new(),
            runtime: VecDeque::new(),
        }
    }

    pub fn send_domain(&mut self, event: DomainEvent) {
        self.domain.push_back(event);
    }
    pub fn send_runtime(&mut self, event: RuntimeEvent) {
        self.runtime.push_back(event);
    }

    pub fn drain_domain(&mut self) -> Vec<DomainEvent> {
        self.domain.drain(..).collect()
    }
    pub fn drain_runtime(&mut self) -> Vec<RuntimeEvent> {
        self.runtime.drain(..).collect()
    }
}

#[derive(Default)]
pub struct Engine<A: Application> {
    pub app: A,
    pub runtime: Option<Runtime>,
    pub bus: EventBus,
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
        
        let Self {app, bus, ..} = self;
        app.init(bus);

        self.runtime = Some(Runtime::new(window.clone()));

        window.request_redraw();
    }

    pub fn tick(&mut self) {
        let Self { app, bus, runtime } = self;

        let Some(runtime) = runtime else {
            return;
        };

        runtime.handle_input(bus);

        runtime.handle_runtime_events(app, bus);

        app.on_update(bus);

        runtime.sync_gpu_assets(app.asset_mgr_mut(), bus);

        runtime.update_ui(app, bus);

        runtime.render(&app.render_data());
    }
}
