use std::sync::Arc;
use std::time::Duration;
use crate::input::Input;
use crate::timer::Timer;

use crate::prelude::*;

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, Event, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

pub struct RunningApp {
    pub window: Arc<Window>,
    pub renderer: Renderer,
    pub uilayer: UiLayer,
    pub is_minimized: bool,
    pub timer: Timer,
    pub input: Input,
}

impl RunningApp {
    fn handle_winit_event(&mut self, event: &Event<()>) {
        // Handle Imgui platform events
        self.uilayer.handle_event(&self.window, event);

        self.input.update_events(&event);
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
}

#[derive(Default)]
pub struct MyApplication {
    app: App,
    runtime: Option<RunningApp>,
    size: winit::dpi::PhysicalSize<u32>,
}

impl MyApplication {
    pub fn new_with_size(width: u32, height: u32) -> Self {
        Self {
            size: winit::dpi::PhysicalSize::new(width, height),
            ..Default::default()
        }
    }
    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = winit::event_loop::EventLoop::new()?;
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        event_loop.run_app(&mut self)?;
        Ok(())
    }
}

pub trait CenterWindow {
    fn try_fit_center_to_monitor(self) -> Self;
}

impl CenterWindow for winit::window::Window {
    fn try_fit_center_to_monitor(self) -> Self {
        if let Some(monitor) = self.current_monitor() {
            let screen_size = monitor.size();
            let window_size = self.inner_size();
            let safe_width = screen_size.width.min(window_size.width);
            let safe_height = screen_size.height.min(window_size.height);

            let x = (screen_size.width.saturating_sub(safe_width)) as f32 / 2.0;
            let y = (screen_size.height.saturating_sub(safe_height)) as f32 / 2.0;
            self.set_outer_position(winit::dpi::PhysicalPosition::new(x, y));
        }
        self
    }
}

impl ApplicationHandler for MyApplication {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("resumed");
        if self.runtime.is_some() {
            println!("Exit");
            return;
        };

        let timer = std::time::Instant::now();
        debug!("App resumed after  {} ms", timer.elapsed().as_millis());

        let window = {
            let wnd = event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_inner_size(self.size)
                        .with_title("App"),
                )
                .map(|w| w.try_fit_center_to_monitor())
                .expect("Failed to crate window");

            Arc::new(wnd)
        };

        self.app.init();
        debug!("App initialized in {} ms", timer.elapsed().as_millis());

        let mut context = imgui::Context::create();
        let renderer = Renderer::new(window.clone(), &mut context, &mut self.app.asset_mgr);
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
        });

        window.request_redraw();
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        let Some(runtime) = &mut self.runtime else {
            return;
        };

        let event = Event::DeviceEvent { device_id, event };
        runtime.handle_winit_event(&event);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let Some(runtime) = &mut self.runtime else {
            return;
        };

        // update timer and input
        runtime.timer.tick_step_iter().for_each(|_dt| {
            runtime.input.clear();
        });

        // Esegue `callback` ogni secondo , in base al clock interno.
        runtime.timer.trigger_every(Duration::from_secs(1), || {
            runtime.renderer.sync_imgui_texture();
            debug!("Sync_with_registry: ");
        });

        runtime.window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(runtime) = &mut self.runtime else {
            return;
        };

        {
            let evt = Event::WindowEvent {
                window_id,
                event: event.clone(),
            };
            runtime.handle_winit_event(&evt);
        }

        match event {
            WindowEvent::CloseRequested => {
                info!("The close button was pressed; stopping");
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    let aspect = size.width.max(1) as f32 / size.height.max(1) as f32;
                    runtime.is_minimized = false;

                    runtime.renderer.resize_frame(size.width, size.height);
                    self.app.camera.set_aspect(aspect);
                } else {
                    runtime.is_minimized = true;
                }
            }
            WindowEvent::RedrawRequested => {
                if runtime.is_minimized {
                    return;
                }

                // Update
                self.app.update_domain_event();
                self.app.update_camera(&runtime.input);
                self.app.update_selected(runtime);
                self.app.update_scene();
                self.app.update_uilayer(runtime);

                // Render
                self.app.render(runtime)
            }
            _ => (),
        }
    }
}
