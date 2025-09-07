use crate::assets::texture_manager::TextureManager;
use crate::input::Input;
use crate::prelude::*;
use crate::renderer::gpu_renderer::DepthTexture;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let timer = std::time::Instant::now();
        println!("App resumed after being paused for {} ms", timer.elapsed().as_millis());
        let size = winit::dpi::LogicalSize::new(1280.0, 720.0);
        let attributes = Window::default_attributes()
            .with_inner_size(size)
            .with_title(format!("App"));
        let window = std::sync::Arc::new(event_loop.create_window(attributes).unwrap());

        self.window = Some(window.clone());
        
        pollster::block_on(Renderer::new(window.clone(), &mut self.resources));
        println!("Renderer initialized in {} ms", timer.elapsed().as_millis());
        
        self.load();
        println!("App initialized in {} ms", timer.elapsed().as_millis());

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
        device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        {
            let mut input = self.resources.get_mut::<Input>().unwrap();
            input.update_device_events(&event);
        }

        //imgui
        if let (Some(window), Some(imgui)) = (&mut self.window, &mut self.imgui) {
            imgui.platform.handle_event::<()>(
                imgui.context.io_mut(),
                &window,
                &winit::event::Event::DeviceEvent { device_id, event },
            );
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let window = match &mut self.window {
            Some(window) => window,
            None => return,
        };

        let mut frame_time = self.clock.elapsed().as_secs_f32() - self.elapsed_time;
        self.frame_time = frame_time * 1000.0;

        while frame_time > 0.0 {
            // Cosa aggiornare dentro questo loop
            // Fisica
            //      Movimento di entità in base a velocità/accelerazione
            //      Collision detection e collision response
            // Gravità e forze varie
            //      Animazioni non legate al rendering
            //      Avanzamento di animazioni di scheletri, timeline di eventi
            // Logica di gioco
            //      AI (aggiornamento percorsi, decisioni)
            //      Stati di missioni/eventi
            // Timer e cooldown
            //      Conti alla rovescia, spawn di nemici, ecc.
            // Sistemi ECS
            //      Tutti i sistemi che dipendono dal tempo e non devono "saltare frame"

            self.delta_time = f32::min(frame_time, self.fixed_timestep);

            self.current_scene
                .update(self.delta_time, &mut self.resources);

            let mut input = self.resources.get_mut::<Input>().unwrap();
            input.clear();

            frame_time -= self.delta_time;
            self.elapsed_time += self.delta_time;
        }

        //imgui
        if let Some(imgui) = &mut self.imgui {
            imgui.platform.handle_event::<()>(
                imgui.context.io_mut(),
                &window,
                &winit::event::Event::AboutToWait,
            );

            let mut registry = self.resources.get_mut::<imgui_tools::ImGuiTextureRegistry>().unwrap();
            let mut renderer = self.resources.get_mut::<imgui_wgpu::Renderer>().unwrap();

            let device = self.resources.get::<wgpu::Device>().unwrap();
            let manager = self.resources.get::<TextureManager>().unwrap();

            // TODO: maybe use an event handler for avoid to sync each frame
            imgui_tools::sync_with_registry(&device, &manager, &mut registry, &mut renderer);
        }

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
                println!("The close button was pressed; stopping");
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
                if self.is_minimized {
                    return;
                }

                // scheduler di update ecs (camera, mesh, etc)
                self.current_scene.schedule.execute(&mut self.current_scene.world, &mut self.resources);

                if let Some(imgui) = &mut self.imgui {
                    let mut scene_world = &mut self.current_scene.world;
                    imgui.update_ui(window, &mut scene_world, &mut self.resources);
                }

                let (frame, view, encoder) = {
                    let device = self.resources.get::<wgpu::Device>().unwrap();
                    let surface = self.resources.get::<wgpu::Surface>().unwrap();
                    let frame = surface
                        .get_current_texture()
                        .expect("Failed to get current texture");
                    let view = frame.texture.create_view(&Default::default());
                    let encoder = device.create_command_encoder(&Default::default());
                    (frame, view, encoder)
                };

                self.resources.insert(encoder);
                self.resources.insert(view);

                // scheduler di rendering (mesh, gui)
                self.render_schedule
                    .execute(&mut self.current_scene.world, &mut self.resources);

                let encoder = self.resources.remove::<wgpu::CommandEncoder>().unwrap();
                let queue = self.resources.get::<wgpu::Queue>().unwrap();

                queue.submit([encoder.finish()]);

                frame.present();
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
