use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, Event, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use crate::app::{Application, RuntimeApp};
use crate::engine::{Engine, RuntimeEvent};

#[derive(Default)]
pub struct MyApplication<A: Application> {
    engine: Engine<A>,
    size: winit::dpi::PhysicalSize<u32>,
}

impl<A: RuntimeApp + Default> MyApplication<A> {
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

impl<A: RuntimeApp> ApplicationHandler for MyApplication<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.engine.resume(event_loop, self.size);
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        let Some(runtime) = &mut self.engine.runtime else {
            return;
        };

        let event = Event::DeviceEvent { device_id, event };
        runtime.handle_winit_event(&event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let Some(runtime) = &mut self.engine.runtime else {
            return;
        };

        if runtime.window.is_minimized().unwrap_or(false) {
            return;
        }

        if runtime.wait_for_exit {
            event_loop.exit();
            return;
        }

        runtime.window.request_redraw();
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(runtime) = &mut self.engine.runtime else {
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
                self.engine
                    .bus
                    .send_runtime(RuntimeEvent::CloseRequested);
            }

            WindowEvent::Resized(size) => {
                self.engine.bus.send_runtime(RuntimeEvent::Resize {
                    width: size.width,
                    height: size.height,
                });
            }
            WindowEvent::RedrawRequested => {
                self.engine.tick();
            }
            WindowEvent::DroppedFile(path) => {
                self.engine.bus.send_runtime(RuntimeEvent::DroppedFile(path));
            }
            _ => (),
        }
    }
}
