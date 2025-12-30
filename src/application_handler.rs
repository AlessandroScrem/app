use std::time::Duration;

use crate::input::Input;
use crate::prelude::*;
use crate::renderer::gpu_manager::GpuManager;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let timer = std::time::Instant::now();
        debug!("App resumed after  {} ms", timer.elapsed().as_millis());

        let window = std::sync::Arc::new(self.create_and_center_window(event_loop));

        Renderer::init(window.clone(), &mut self.resources);
        debug!("Renderer initialized in {} ms", timer.elapsed().as_millis());

        self.imgui = Some(ui::ImguiState::new(&window, &mut self.resources));

        self.init();
        debug!("App initialized in {} ms", timer.elapsed().as_millis());

        self.window = Some(window.clone());
        window.request_redraw();
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        {
            let mut input = self.resources.get_mut::<Input>().unwrap();
            input.update_device_events(&event);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let window = match &mut self.window {
            Some(window) => window,
            None => return,
        };

        // update timer and input
        self.timer.tick_step_iter().for_each(|_dt| {
            self.resources.get_mut::<Input>().unwrap().clear();
        });

        // Esegue `callback` ogni secondo , in base al clock interno.
        self.timer.trigger_every(Duration::from_secs(1), || {
            self.update_schedule
                .execute(&mut self.current_scene.world, &mut self.resources);
        });

        window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let window = match &mut self.window {
            Some(window) => window,
            None => return,
        };

        let mut imgui_capture_events = false;
        if let Some(imgui) = &mut self.imgui {
            imgui.platform.handle_event::<()>(
                imgui.context.io_mut(),
                window,
                &winit::event::Event::WindowEvent {
                    window_id,
                    event: event.clone(),
                },
            );
            imgui_capture_events =
                imgui.context.io().want_capture_mouse || imgui.context.io().want_capture_keyboard;
        }

        if !imgui_capture_events {
            let mut input = self.resources.get_mut::<Input>().unwrap();
            input.update_window_events(&event);
        }

        match event {
            WindowEvent::CloseRequested => {
                info!("The close button was pressed; stopping");
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    self.is_minimized = false;
                    resize_resources(&mut self.resources, size.width, size.height);
                } else {
                    self.is_minimized = true;
                }
            }
            WindowEvent::RedrawRequested => {
                self.update_scene();
                self.render();
            }
            _ => (),
        }
    }
}

fn resize_resources(resources: &mut legion::Resources, width: u32, height: u32) {
    {
        let mut surface_config = resources.get_mut::<wgpu::SurfaceConfiguration>().unwrap();
        surface_config.width = width;
        surface_config.height = height;

        let surface = resources.get::<wgpu::Surface>().unwrap();
        let device = resources.get::<wgpu::Device>().unwrap();

        surface.configure(&device, &surface_config);
    }
    // resize gpu_manager
    {
        let device = resources.get::<wgpu::Device>().unwrap();
        let mut gpu_manager = resources.get_mut::<GpuManager>().unwrap();
        gpu_manager.resize_frame(&device, width, height);
    }
}
