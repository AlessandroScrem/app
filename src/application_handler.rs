use crate::input::Input;
use crate::prelude::*;

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, ElementState, MouseButton, MouseScrollDelta, WindowEvent};
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
            &self.camera,
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

        let renderer = self.renderer.as_mut().unwrap();

        if renderer.handle_input(&event).consumed {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
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
                self.current_scene
                    .schedule
                    .execute(&mut self.current_scene.world, &mut self.resources);

                // 👇 Estrai temporaneamente, lo rimetteremo poi
                let mut renderer = self.renderer.take().unwrap();

                renderer.update_camera_buffer(&self.camera);

                renderer.update(self.delta_time);

                let mut gui_callback = |ctx: &egui::Context, ui: &mut egui::Ui| {
                    self.update_gui(ctx, ui);
                };

                let _ = renderer.render(&mut gui_callback);

                renderer.get_window().request_redraw();

                // 👈 Rimetti il renderer dentro self
                self.renderer = Some(renderer);
            }
            _ => (),
        }
    }
}
