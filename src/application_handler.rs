use crate::input::Input;
use crate::prelude::*;

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = std::sync::Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        self.window = Some(window.clone());

        self.renderer = Some(pollster::block_on(Renderer::new(
            window.clone(),
            &mut self.resources,
            &self.current_scene.world,
        )));
        self.load();

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
        let mut frame_time = self.clock.elapsed().as_secs_f32() - self.elapsed_time;
        self.frame_time = frame_time * 1000.0;

        while frame_time > 0.0 {
            self.delta_time = f32::min(frame_time, self.fixed_timestep);

            self.current_scene
                .update(self.delta_time, &mut self.resources);

            {
                let mut input = self.resources.get_mut::<Input>().unwrap();
                input.clear();
            }

            frame_time -= self.delta_time;
            self.elapsed_time += self.delta_time;
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        {
            let mut input = self.resources.get_mut::<Input>().unwrap();
            input.update_window_events(&event);
        }

        {
            if let Some(window) = &self.window {
                let mut egui_renderer = self
                    .resources
                    .get_mut::<egui_tools::EguiRenderer>()
                    .unwrap();
                if egui_renderer.handle_input(&window, &event).consumed {
                    return;
                }
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    let surface = self.resources.get_mut::<wgpu::Surface>().unwrap();
                    let mut surface_config = self
                        .resources
                        .get_mut::<wgpu::SurfaceConfiguration>()
                        .unwrap();
                    let device = self.resources.get_mut::<wgpu::Device>().unwrap();

                    surface_config.width = size.width;
                    surface_config.height = size.height;
                    surface.configure(&device, &surface_config);

                    use legion::IntoQuery;
                    let mut query = <legion::Write<Camera>>::query();
                    for camera in query.iter_mut(&mut self.current_scene.world) {
                        camera.set_aspect(size.width as f32 / size.height as f32);
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = &self.renderer {
                    {
                        let mut resources = &mut self.resources;
                        self.current_scene
                            .schedule
                            .execute(&mut self.current_scene.world, &mut resources);
                    }
                    {
                        // update camera uniform buffer
                        let mut camera_uniform = self.resources.get_mut::<CameraUniform>().unwrap();
                        let camera_buffer = self.resources.get::<wgpu::Buffer>().unwrap();
                        let queue = self.resources.get_mut::<wgpu::Queue>().unwrap();
                        use legion::IntoQuery;
                        let mut query = <legion::Read<Camera>>::query();
                        for camera in query.iter(&self.current_scene.world) {
                            camera_uniform.update_view_proj(camera);
                        }

                        queue.write_buffer(
                            &camera_buffer,
                            0,
                            bytemuck::cast_slice(&[camera_uniform.clone()]),
                        );
                    }

                    let _ = renderer.render(&mut self.resources);

                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }
            _ => (),
        }
    }
}
