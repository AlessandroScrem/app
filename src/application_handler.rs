use crate::assets::asset_manager::AssetManager;
use crate::input::Input;
use crate::timer::Timer;
use std::sync::Arc;

use crate::prelude::*;

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, Event, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

pub trait Application{
    fn init(&mut self);
    fn update(&mut self, runtime: &mut RunningApp);
    fn render(&mut self, runtime: &mut RunningApp);
    fn on_resize(&mut self, width: u32, height: u32);
    fn on_close(&mut self);
}

impl Application for App {
    fn init(&mut self) {
        let timer = std::time::Instant::now();

        self.domain_events
            .queue
            .push_back(DomainEvent::Assets(AssetEvent::LoadGltf(
                "./assets/Lantern/Lantern.gltf".into(),
            )));
        let hdrpath = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/core/newport_loft.hdr");
        let hdr_id = self
            .asset_mgr
            .textures
            .from_file(hdrpath, renderer::TextureUsage::HDR16);
        self.asset_mgr.skybox = assets::asset_manager::SkyboxHandle::new(hdr_id);

        crate::entities::light::create(&mut self.current_scene.world, &self.resources);

        self.current_scene.schedule = crate::systems::create_current_scene_schedule_builder();

        debug!("App loader took {} ms", timer.elapsed().as_millis());
    }
    fn update(&mut self, runtime: &mut RunningApp) {
        // Esegue `callback` ogni secondo , in base al clock interno.
        runtime.timer
            .trigger_every(std::time::Duration::from_secs(1), || {
                runtime.renderer.sync_imgui_texture();
                debug!("Sync_with_registry: ");
            });

        self.update_domain_event();
        self.update_camera(&runtime.input);
        self.update_selected(runtime);
        self.update_scene();
        self.update_uilayer(runtime);
    }

    fn on_resize(&mut self, width: u32, height: u32) {
        
        let aspect = width.max(1) as f32 / height.max(1) as f32;
        self.camera.set_aspect(aspect);
    }
    fn render(&mut self, runtime: &mut RunningApp) {
        runtime.renderer.render(
            &self.asset_mgr,
            &self.current_scene.world,
            &mut self.resources,
            &self.camera,
            &self.globals,
            self.selected,
            &runtime.input,
            runtime.uilayer.get_draw_data(),
        );
    }
    fn on_close(&mut self) {
        info!("The close button was pressed; App stopping");
    }
}

pub trait HasAssetMgr {
    fn asset_mgr_mut(&mut self) -> &mut AssetManager;
}

impl HasAssetMgr for App {
    fn asset_mgr_mut(&mut self) -> &mut AssetManager {
        &mut self.asset_mgr
    }
}

#[derive(Default)]
pub struct Engine<A: Application> {
    pub app: A,
    pub runtime: Option<RunningApp>,
}

impl <A: Application + HasAssetMgr> Engine<A> {
    pub fn resume(&mut self, event_loop: &ActiveEventLoop, size: PhysicalSize<u32> ) {
        if self.runtime.is_some() {
            return;
        };

        let timer = std::time::Instant::now();
        debug!("App resumed after  {} ms", timer.elapsed().as_millis());

        let window = {
            let wnd = event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_inner_size(size)
                        .with_title("App"),
                )
                .map(|w| w.try_fit_center_to_monitor())
                .expect("Failed to crate window");

            Arc::new(wnd)
        };

        self.app.init();
        debug!("App initialized in {} ms", timer.elapsed().as_millis());

        let mut context = imgui::Context::create();
        let asset_mgr = self.app.asset_mgr_mut();
        let renderer = Renderer::new(window.clone(), &mut context, asset_mgr);
        let adapter_string = renderer.get_adapter_string();
        let uilayer = UiLayer::new(&window, context, adapter_string);

        debug!("Renderer initialized in {} ms", timer.elapsed().as_millis());

        self.runtime = Some(RunningApp {
            window: window.clone(),
            input: Input::new(),
            renderer,
            uilayer,
            is_minimized: false,
            timer: Timer::new(),
            events: Vec::new(),
        });

        window.request_redraw();

    }
}

pub enum RuntimeEvent {
    Resize { width: u32, height: u32 },
    CloseRequested,
}

pub struct RunningApp {
    pub window: Arc<Window>,
    pub renderer: Renderer,
    pub uilayer: UiLayer,
    pub is_minimized: bool,
    pub timer: Timer,
    pub input: Input,
    pub events: Vec<RuntimeEvent>,
}

impl RunningApp {
    fn handle_winit_event(&mut self, event: &Event<()>) {
        // Handle Imgui platform events
        self.uilayer.handle_event(&self.window, event);

        // Handle Input
        match event {
            Event::WindowEvent { .. } | Event::DeviceEvent { .. } => {
                if !self.uilayer.want_capture_mouse() {
                    self.input.update_events(&event);
                }
            }
            _ => {}
        }
    }

    fn tick<A: Application>(&mut self, app: &mut A) {
        if self.is_minimized {
            return;
        }

        let events = std::mem::take(&mut self.events);
        for event in events {
            self.handle_runtime_event(app, event);
        }

        // let dt = self.timer.tick();
        app.update(self);

        // Render
        app.render(self);

        // Clear Input
        self.input.clear();
    }

    fn handle_runtime_event<A:Application>(&mut self, app: &mut A, event: RuntimeEvent) {
        match event {
            RuntimeEvent::Resize { width, height } => {
                if width > 0 && height > 0 {
                    self.is_minimized = false;
                    self.renderer.resize_frame(width, height);
                    app.on_resize(width, height);
                } else {
                    self.is_minimized = true;
                }
            }
            RuntimeEvent::CloseRequested => {
                app.on_close();
            }
        }
    }
    
}

#[derive(Default)]
pub struct MyApplication<A: Application> {
    engine: Engine<A>,
    size: winit::dpi::PhysicalSize<u32>,
}

impl <A: Application + Default + HasAssetMgr> MyApplication<A> {
    pub fn new_with_size(width: u32, height: u32) -> Self {
        Self {
            size: winit::dpi::PhysicalSize::new(width, height),
            ..Default::default()
        }
    }
    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = winit::event_loop::EventLoop::new()?;
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        event_loop.run_app(&mut self)?;
        Ok(())
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

impl <A: Application + HasAssetMgr> ApplicationHandler for MyApplication<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("resumed");
        self.engine.resume(event_loop, self.size);
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        let Some(runtime) = &mut self.engine.runtime else {
            return;
        };

        let event = Event::DeviceEvent { device_id, event };
        runtime.handle_winit_event(&event);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let Some(runtime) = &mut self.engine.runtime else {
            return;
        };
        if runtime.is_minimized {
            return;
        }

        runtime.window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(runtime) = &mut self.engine.runtime else {
            return;
        };

        {
            let evt = Event::WindowEvent {
                window_id,
                event: event.clone(),
            };
            runtime.handle_winit_event(&evt);
        }

        match event {
            WindowEvent::CloseRequested => {
                runtime.events.push(RuntimeEvent::CloseRequested);
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                runtime.events.push(RuntimeEvent::Resize {
                    width: size.width,
                    height: size.height,
                });
            }
            WindowEvent::RedrawRequested => {
                runtime.tick(&mut self.engine.app);
            }
            _ => (),
        }
    }
}
