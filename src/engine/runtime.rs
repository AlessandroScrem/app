use std::sync::Arc;

use winit::{event::Event, window::Window};
use super::{RuntimeEvent};
use crate::app::Application;
use crate::Renderer;
use crate::UiLayer;
use crate::input::Input;
use crate::timer::Timer;

pub(crate) struct RunningApp {
    pub(crate) window: Arc<Window>,
    pub(crate) renderer: Renderer,
    pub(crate) uilayer: UiLayer,
    pub(crate) is_minimized: bool,
    pub(crate) timer: Timer,
    pub(crate) input: Input,
    pub(crate) events: Vec<RuntimeEvent>,
}

impl RunningApp {
    pub(crate) fn handle_winit_event(&mut self, event: &Event<()>) {
        // Handle Imgui platform events
        self.uilayer.handle_event(&self.window, event);

        // Handle Input
        match event {
            Event::WindowEvent { .. } | Event::DeviceEvent { .. } => {
                if !self.uilayer.want_capture_mouse() {
                    self.input.update_events(&event);
                }
            }
            _ => {}
        }
    }

    pub(crate) fn tick<A: Application>(&mut self, app: &mut A) {
        if self.is_minimized {
            return;
        }

        let events = std::mem::take(&mut self.events);
        for event in events {
            self.handle_runtime_event(app, event);
        }

        // let dt = self.timer.tick();
        app.update(self);

        // Render
        app.render(self);

        // Clear Input
        self.input.clear();
    }

    fn handle_runtime_event<A:Application>(&mut self, app: &mut A, event: RuntimeEvent) {
        match event {
            RuntimeEvent::Resize { width, height } => {
                if width > 0 && height > 0 {
                    self.is_minimized = false;
                    self.renderer.resize_frame(width, height);
                    app.on_resize(width, height);
                } else {
                    self.is_minimized = true;
                }
            }
            RuntimeEvent::CloseRequested => {
                app.on_close();
            }
        }
    }
    
}