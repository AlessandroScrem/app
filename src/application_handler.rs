use crate::input::Input;
use crate::prelude::*;
use crate::renderer::gpu_renderer::DepthTexture;

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

        pollster::block_on(Renderer::new(window.clone(), &mut self.resources));
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

        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    resize_resources(&mut self.resources, size.width, size.height);

                    // update camera aspect ratio
                    use legion::IntoQuery;
                    let mut query = <&mut Camera>::query();
                    query
                        .iter_mut(&mut self.current_scene.world)
                        .next()
                        .map(|camera| camera.set_aspect(size.width as f32 / size.height as f32));
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(window) = &self.window {
                    let mut resources = &mut self.resources;
                    self.current_scene
                        .schedule
                        .execute(&mut self.current_scene.world, &mut resources);

                    self.render_schedule
                        .execute(&mut self.current_scene.world, &mut resources);

                    window.request_redraw();
                }
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

        let surface = resources.get_mut::<wgpu::Surface>().unwrap();
        let device = resources.get_mut::<wgpu::Device>().unwrap();

        surface.configure(&device, &surface_config);
    }
    // resize depth texture
    {
        let depth_texture = {
            let device = resources.get_mut::<wgpu::Device>().unwrap();
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
