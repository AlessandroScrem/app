mod renderer;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use renderer::Renderer;
use std::sync::Arc;

pub struct App {
    renderer: Option<Renderer>,
    last_render_time: instant::Instant,
}

impl Default for App {
    fn default() -> Self {
        Self {
            renderer: None,
            last_render_time: instant::Instant::now(),
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
        let renderer = pollster::block_on(Renderer::new(window.clone()));
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
            WindowEvent::RedrawRequested => {
                let now = instant::Instant::now();
                let dt = now - self.last_render_time;
                self.last_render_time = now;

                renderer.update(dt);

                let _ = renderer.render();
                renderer.get_window().request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                println!("Keyboard input: {:?}", event.physical_key);
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
            _ => (),
        }
    }
}