use crate::prelude::*;

use crate::camera;

use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::KeyEvent;
use winit::event::WindowEvent;
use winit::event::{DeviceEvent, ElementState, MouseButton, MouseScrollDelta};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::PhysicalKey;
use winit::window::Window;
use winit::window::WindowId;


pub struct App {
    renderer: Option<Renderer>,
    last_render_time: instant::Instant,
    camera: camera::Camera,
    mouse_pressed: Option<MouseButton>,
}

impl Default for App {
    fn default() -> Self {
        const FOV: cgmath::Deg<f32> = cgmath::Deg::<f32>(45.0);
        let camera = camera::Camera::new(FOV, 1.0, 0.1, 100.0);

        Self {
            renderer: None,
            last_render_time: instant::Instant::now(),
            camera,
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
        let renderer = pollster::block_on(Renderer::new(window.clone(), &self.camera));
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

        if renderer.handle_input(&event).consumed {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(_key),
                        ..
                    },
                ..
            } => {}

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

                renderer.update_camera_buffer(&self.camera);

                renderer.update(dt);

                let _ = renderer.render(&mut self.camera);
                renderer.get_window().request_redraw();
            }
            _ => (),
        }
    }
    
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
        let _ = (event_loop, cause);
    }
    
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: ()) {
        let _ = (event_loop, event);
    }
    
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }
    
    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }
    
    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }
    
    fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }
}
