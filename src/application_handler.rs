use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

use crate::app::AppTimer;
use crate::input::Input;

use crate::picking::PickObject;
use crate::prelude::ui::ImguiLayer;
use crate::{DomainEvent, prelude::*, renderer};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, Event, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

pub struct EventQueue {
    pub queue: VecDeque<Event<()>>,
}

pub struct RunningApp {
    pub window: Arc<Window>,
    pub renderer: Renderer,
    pub imgui: ui::ImguiLayer,
    pub is_minimized: bool,
    pub timer: AppTimer,
    pub event_queue: EventQueue,
    pub input: Input,
}

impl RunningApp {
    pub fn input_update(&mut self) {
        while let Some(event) = self.event_queue.queue.pop_front() {
            self.imgui.platform.handle_event::<()>(
                self.imgui.context.io_mut(),
                &self.window,
                &event,
            );

            let io = self.imgui.context.io();

            match &event {
                Event::DeviceEvent { .. } | Event::WindowEvent { .. } => {
                    if !io.want_capture_mouse {
                        self.input.update_events(&event);
                    }
                }
                _ => {}
            }
        }
    }
}

#[derive(Default)]
pub struct MyApplication {
    app: App,
    runtime: Option<RunningApp>,
    size: winit::dpi::PhysicalSize<u32>,
}

impl MyApplication {
    pub fn new_with_size(width: u32, height: u32) -> Self {
        Self {
            size: winit::dpi::PhysicalSize::new(width, height),
            ..Default::default()
        }
    }
    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = winit::event_loop::EventLoop::new().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        event_loop.run_app(&mut self)?;
        Ok(())
    }
}

fn update_domain_event(runtime: &mut RunningApp, app: &mut App) {
    let world = &mut app.current_scene.world;
    let gpu_view = &mut runtime.renderer.get_gpu_view();
    let mut gpu = crate::assets::mesh::GpuDevice {
        device: gpu_view.device,
        gpu_mgr: gpu_view.gpu_mgr,
        mat_mgr: gpu_view.mat_mgr,
        mesh_mgr: gpu_view.mesh_mgr,
        tex_mgr: gpu_view.texture_mgr,
    };

    while let Some(event) = app.domain_events.queue.pop_front() {
        match event {
            DomainEvent::RemoveEntity(entity) => {
                crate::entities::remove_from_root(entity, world);
            }
            DomainEvent::LoadGltf(path) => {
                println!("Fired load gltf");
                let loaded = crate::assets::mesh::load(path).unwrap();
                let gpu_scene = crate::assets::mesh::upload_scene_to_gpu(&loaded, &mut gpu);
                crate::assets::mesh::spawn_scene(world, &loaded, &gpu_scene);
            }
            DomainEvent::AddParent(entity) => {
                crate::entities::add_parent(entity, world);
            }
        }
    }
}

fn update_camera(input: &mut Input, camera: &mut Camera) {
    // move away from here

    if input.is_mouse_button_down(crate::input::MouseButton::Left) {
        let delta = (input.mouse_delta.x as f64, input.mouse_delta.y as f64);
        camera.orbit(delta);
    }

    if input.is_mouse_button_down(crate::input::MouseButton::Middle) {
        let delta = (input.mouse_delta.x as f64, input.mouse_delta.y as f64);
        camera.pan(delta);
    }

    if let Some(delta) = input.mouse_wheel_movement {
        camera.zoom(delta.y);
    }
}

fn update_selcted(input: &Input, pickobject: &mut PickObject) {
    // update hovered entity_id from buffer
    use crate::input::MouseButton;
    use winit::keyboard::{Key, NamedKey};
    if input.is_cursor_moved() {
        pickobject.apply();
    }
    if input.is_mouse_button_pressed(MouseButton::Left)
        && input.is_key_down(Key::Named(NamedKey::Alt))
    {
        pickobject.select_hovered();
    }
}

pub trait CenterWindow {
    fn try_fit_center_to_monitor(self) -> Self;
}

impl CenterWindow for winit::window::Window {
    fn try_fit_center_to_monitor(self) -> Self {
        if let Some(monitor) = self.current_monitor() {
            let screen_size = monitor.size();
            let window_size = self.inner_size();
            let safe_width = screen_size.width.min(window_size.width);
            let safe_height = screen_size.height.min(window_size.height);

            let x = (screen_size.width.saturating_sub(safe_width)) as f32 / 2.0;
            let y = (screen_size.height.saturating_sub(safe_height)) as f32 / 2.0;
            self.set_outer_position(winit::dpi::PhysicalPosition::new(x, y));
        }
        self
    }
}

impl ApplicationHandler for MyApplication {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("resumed");
        if self.runtime.is_some() {
            println!("Exit");
            return;
        };

        let timer = std::time::Instant::now();
        debug!("App resumed after  {} ms", timer.elapsed().as_millis());

        let window = {
            let wnd = event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_inner_size(self.size)
                        .with_title("App"),
                )
                .map(|w| w.try_fit_center_to_monitor())
                .expect("Failed to crate window");

            Arc::new(wnd)
        };

        let renderer = Renderer::new(window.clone());
        debug!("Renderer initialized in {} ms", timer.elapsed().as_millis());

        let imgui = ImguiLayer::new(
            &window,
            &renderer.device,
            &renderer.queue,
            renderer.surface_config.format,
        );

        self.app.init();
        debug!("App initialized in {} ms", timer.elapsed().as_millis());

        self.runtime = Some(RunningApp {
            window: window.clone(),
            event_queue: EventQueue {
                queue: VecDeque::new(),
            },
            input: Input::new(),
            renderer,
            imgui,
            is_minimized: false,
            timer: AppTimer::new(),
        });

        window.request_redraw();
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        let Some(runtime) = &mut self.runtime else {
            return;
        };

        {
            let event = Event::DeviceEvent { device_id, event };
            runtime.event_queue.queue.push_back(event);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let Some(runtime) = &mut self.runtime else {
            return;
        };

        // update timer and input
        runtime.timer.tick_step_iter().for_each(|_dt| {
            runtime.input.clear();
        });

        // Esegue `callback` ogni secondo , in base al clock interno.
        runtime.timer.trigger_every(Duration::from_secs(1), || {
            let gpu = runtime.renderer.get_gpu_view();
            runtime
                .imgui
                .sync_with_registry(&gpu.device, gpu.texture_mgr);
            debug!("Sync_with_registry: ");
        });

        update_domain_event(runtime, &mut self.app);

        runtime.window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(runtime) = &mut self.runtime else {
            return;
        };

        runtime.event_queue.queue.push_back(Event::WindowEvent {
            window_id,
            event: event.clone(),
        });

        match event {
            WindowEvent::CloseRequested => {
                info!("The close button was pressed; stopping");
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    let aspect = size.width.max(1) as f32 / size.height.max(1) as f32;
                    runtime.is_minimized = false;
                    runtime.renderer.resize_resources(size.width, size.height);
                    self.app.camera.set_aspect(aspect);
                } else {
                    runtime.is_minimized = true;
                }
            }
            WindowEvent::RedrawRequested => {
                if runtime.is_minimized {
                    return;
                }

                // Update
                runtime.input_update();
                update_camera(&mut runtime.input, &mut self.app.camera);
                update_selcted(&runtime.input, &mut runtime.renderer.pickobject);

                self.app.update_scene();
                self.app.imgui_update(
                    &mut runtime.imgui,
                    &runtime.window,
                    &runtime.renderer,
                );

                // Render
                self.app
                    .render(&mut runtime.renderer, &mut runtime.imgui, &runtime.input)
            }
            _ => (),
        }
    }
}
