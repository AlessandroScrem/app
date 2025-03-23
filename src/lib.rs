mod camera;
mod renderer;

mod prelude {
    pub use crate::camera::*;
    pub use crate::renderer::*;
    pub use std::sync::Arc;
    pub use winit::application::ApplicationHandler;
    pub use winit::event::{KeyEvent, WindowEvent};
    pub use winit::event_loop::ActiveEventLoop;
    pub use winit::keyboard::PhysicalKey;
    pub use winit::window::{Window, WindowId};
}

use prelude::*;

pub struct App {
    renderer: Option<Renderer>,
    last_render_time: instant::Instant,
    camera: camera::Camera,
    projection: camera::Projection,
    camera_controller: camera::CameraController,
}

impl Default for App {
    fn default() -> Self {
        let camera = camera::Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let projection = camera::Projection::new(800, 600, cgmath::Deg(45.0), 0.1, 100.0);
        let camera_controller = camera::CameraController::new(4.0, 0.4);

        Self {
            renderer: None,
            last_render_time: instant::Instant::now(),
            camera,
            projection,
            camera_controller,
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
            &self.projection,
        ));
        self.renderer = Some(renderer);

        window.request_redraw();
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
                self.camera_controller.process_keyboard(key, state);
            }
            WindowEvent::MouseInput { button, .. } => {
                println!("Mouse input: {:?}", button);
            }
            WindowEvent::CursorMoved { position, .. } => {
                println!("Cursor moved: {:?}", position);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                println!("Mouse wheel: {:?}", delta);
            }
            WindowEvent::Resized(size) => {
                renderer.resize(size);
                println!("Resized: {:?}", size);
            }
            WindowEvent::RedrawRequested => {
                let now = instant::Instant::now();
                let dt = now - self.last_render_time;
                self.last_render_time = now;

                self.camera_controller.update_camera(&mut self.camera, dt);
                renderer.update_camera_buffer(&self.camera, &self.projection);

                renderer.update(dt);

                let _ = renderer.render();
                renderer.get_window().request_redraw();
            }
            _ => (),
        }
    }
}
