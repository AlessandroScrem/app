use crate::prelude::*;

use winit::application::ApplicationHandler;
use winit::event::{WindowEvent, KeyEvent, DeviceEvent, ElementState, MouseButton, MouseScrollDelta};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};

struct DisplayPoint3(cgmath::Point3<f32>);

impl std::fmt::Display for DisplayPoint3 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({:.2}, {:.2}, {:.2})", self.0.x, self.0.y, self.0.z)
    }
}

pub struct App {
    renderer: Option<Renderer>,
    last_render_time: instant::Instant,
    camera: Camera,
    mouse_pressed: Option<MouseButton>,
}

impl Default for App {
    fn default() -> Self {
        const FOV: cgmath::Deg<f32> = cgmath::Deg::<f32>(45.0);
        let camera = Camera::new(FOV, 1.0, 0.1, 100.0);

        Self {
            renderer: None,
            last_render_time: instant::Instant::now(),
            camera,
            mouse_pressed: None,
        }
    }
}
impl App {
    pub fn update_gui(&mut self, ui: &mut egui::Ui) {
        let mut picked_path: Option<String> = None;
        if ui.button("Open file…").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                picked_path = Some(path.display().to_string());
            }
        }

        if let Some(picked_path) = &picked_path {
            ui.horizontal(|ui| {
                ui.label("Picked file:");
                ui.monospace(picked_path);
            });
        }

        ui.label(format!(
            "Camera position: {}",
            DisplayPoint3(self.camera.get_position())
        ));
        let mut fov: f32 = self.camera.get_fov().0;
        if ui
            .add(egui::Slider::new(&mut fov, 0.1..=179.0).text("Fov"))
            .changed()
        {
            self.camera.set_fov(cgmath::Deg(fov));
        };
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = std::sync::Arc::new(
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
                    println!("Resized: {:?}", size);
                }
            }
            WindowEvent::RedrawRequested => {
                let now = instant::Instant::now();
                let dt = now - self.last_render_time;
                self.last_render_time = now;

                // 👇 Estrai temporaneamente, lo rimetteremo poi
                let mut renderer = self.renderer.take().unwrap();

                renderer.update_camera_buffer(&self.camera);

                renderer.update(dt);

                let mut gui_callback = |ui: &mut egui::Ui| {
                    self.update_gui(ui);
                };

                let _ = renderer.render(&mut gui_callback);

                renderer.get_window().request_redraw();

                // 👈 Rimetti il renderer dentro self
                self.renderer = Some(renderer);
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
