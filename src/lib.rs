mod camera;
mod renderer;

mod prelude {
    pub use crate::camera::*;
    pub use crate::renderer::*;
    pub use std::sync::Arc;
    pub use winit::event::{KeyEvent, WindowEvent};
    pub use winit::keyboard::PhysicalKey;
    pub use winit::window::{Window, WindowId};
}

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, ElementState, MouseButton, MouseScrollDelta};
use winit::event_loop::ActiveEventLoop;

use prelude::*;

pub struct App {
    renderer: Option<Renderer>,
    last_render_time: instant::Instant,
    camera: camera::Camera,
    // projection: camera::Projection,
    // camera_controller: camera::CameraController,
    mouse_pressed: Option<MouseButton>,
}

impl Default for App {
    fn default() -> Self {
        const FOV: cgmath::Deg<f32> = cgmath::Deg::<f32>(45.0);
        let camera = camera::Camera::new(FOV, 1.0, 0.1, 100.0);
        // let projection = camera::Projection::new(800, 600, cgmath::Deg(45.0), 0.1, 100.0);
        // let camera_controller = camera::CameraController::new(4.0, 0.4);

        Self {
            renderer: None,
            last_render_time: instant::Instant::now(),
            camera,
            // projection,
            // camera_controller,
            mouse_pressed: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
        let renderer = pollster::block_on(Renderer::new(
            window.clone(),
            &self.camera,
            // &self.projection,
        ));
        self.renderer = Some(renderer);

        window.request_redraw();
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        match event {
            DeviceEvent::MouseMotion { delta } => match self.mouse_pressed {
                Some(MouseButton::Left) => {
                    self.camera.orbit(delta);
                }
                Some(MouseButton::Middle) => {
                    self.camera.pan(delta);
                }
                _ => (),
            },
            _ => (),
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let renderer = self.renderer.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state,
                        ..
                    },
                ..
            } => {
                // self.camera_controller.process_keyboard(key, state);
            }

            WindowEvent::MouseInput { button, state, .. } => {
                if state == ElementState::Pressed {
                    self.mouse_pressed = Some(button)
                } else {
                    self.mouse_pressed = None;
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll_y = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                };
                // self.camera_controller.process_scroll(&delta);
                self.camera.zoom(scroll_y);
            }
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    renderer.resize(size);
                    self.camera
                        .set_aspect(size.width as f32 / size.height as f32);
                }
                println!("Resized: {:?}", size);
            }
            WindowEvent::RedrawRequested => {
                let now = instant::Instant::now();
                let dt = now - self.last_render_time;
                self.last_render_time = now;

                // self.camera_controller.update_camera(&mut self.camera, dt);
                renderer.update_camera_buffer(&self.camera /* , &self.projection */);

                renderer.update(dt);

                let _ = renderer.render();
                renderer.get_window().request_redraw();
            }
            _ => (),
        }
    }
}
