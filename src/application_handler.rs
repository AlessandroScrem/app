use std::sync::Arc;
use std::time::Duration;

use crate::input::Input;
use crate::prelude::*;
use crate::renderer::gpu_manager::GPUResourceManager;
use crate::renderer::hdr_frame::IDTexture;
use crate::renderer::{gpu_renderer::DepthTexture, hdr_frame::HdrFrame};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let timer = std::time::Instant::now();
        info!(
            "App resumed after being paused for {} ms",
            timer.elapsed().as_millis()
        );

        let window = self.create_and_center_window(event_loop);
        self.window = Some(window.clone());

        Renderer::init(window.clone(), &mut self.resources);
        info!("Renderer initialized in {} ms", timer.elapsed().as_millis());

        self.load();
        info!("App initialized in {} ms", timer.elapsed().as_millis());

        window.request_redraw();
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: ()) {
        //imgui
        if let (Some(window), Some(imgui)) = (&mut self.window, &mut self.imgui) {
            imgui.platform.handle_event::<()>(
                imgui.context.io_mut(),
                &window,
                &winit::event::Event::UserEvent(event),
            );
        }
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
        self.timer.tick_step_iter().for_each(|dt| {
            trace!("dt: {dt}");
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
    // resize hdr texture
    {
        resources.insert({
            let device = resources.get::<wgpu::Device>().unwrap();
            let gpu_resource_manager = resources.get::<Arc<GPUResourceManager>>().unwrap();
            HdrFrame::new(
                &device,
                &gpu_resource_manager,
                winit::dpi::PhysicalSize::new(width, height),
            )
        });
    }
    // resize entity_id_texture
    {
        resources.insert({
            let device = resources.get::<wgpu::Device>().unwrap();
            let gpu_resource_manager = resources.get::<Arc<GPUResourceManager>>().unwrap();
            IDTexture::new(
                &device,
                &gpu_resource_manager,
                winit::dpi::PhysicalSize::new(width, height),
            )
        });
    }

    // resize depth texture
    {
        let depth_texture = {
            let device = resources.get::<wgpu::Device>().unwrap();
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some("depth_texture"),
                size: wgpu::Extent3d {
                    width: width,
                    height: height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
        };
        let depth_view = depth_texture.create_view(&Default::default());
        resources.insert(DepthTexture(depth_view));
    }
}
