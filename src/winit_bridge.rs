use std::sync::Arc;

use imgui::{Context, Ui};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{DeviceEvent, ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{
        CursorIcon, ResizeDirection, Window, WindowAttributes, WindowId,
    },
};

use crate::app::RuntimeApp;
use crate::engine::{Engine, RuntimeEvent};

pub(crate) type WindowHandle = Arc<Window>;
pub(crate) type WinitEvent<T> = Event<T>;

pub(crate) struct WinitUiPlatform {
    platform: WinitPlatform,
}

impl WinitUiPlatform {
    pub(crate) fn new(context: &mut Context, window: &WindowHandle) -> Self {
        let mut platform = WinitPlatform::new(context);
        platform.attach_window(
            context.io_mut(),
            window,
            HiDpiMode::Default,
        );
        Self { platform }
    }

    pub(crate) fn handle_event<T>(&mut self, context: &mut Context, window: &WindowHandle, event: &Event<T>) {
        self.platform
            .handle_event::<T>(context.io_mut(), window, event);
    }

    pub(crate) fn prepare_frame(&mut self, context: &mut Context, window: &WindowHandle) {
        self.platform
            .prepare_frame(context.io_mut(), window)
            .expect("failed to prepare frame");
    }

    pub(crate) fn prepare_render(&mut self, ui: &Ui, window: &WindowHandle) {
        self.platform.prepare_render(ui, window);
    }
}

#[derive(Default)]
pub(crate) struct MyApplication<A: RuntimeApp + Default> {
    engine: Engine<A>,
    size: PhysicalSize<u32>,
    cursor_position: Option<PhysicalPosition<f64>>,
}

impl<A: RuntimeApp + Default> MyApplication<A> {
    pub fn new_with_size(width: u32, height: u32) -> Self {
        Self {
            size: PhysicalSize::new(width, height),
            cursor_position: None,
            ..Default::default()
        }
    }

    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(&mut self)?;
        Ok(())
    }
}

pub trait CenterWindow {
    fn try_fit_center_to_monitor(self) -> Self;
}

impl CenterWindow for WindowHandle {
    fn try_fit_center_to_monitor(self) -> Self {
        if let Some(monitor) = self.current_monitor() {
            let screen_size = monitor.size();
            let window_size = self.inner_size();
            let safe_width = screen_size.width.min(window_size.width);
            let safe_height = screen_size.height.min(window_size.height);
            let x = (screen_size.width.saturating_sub(safe_width)) as f32 / 2.0;
            let y = (screen_size.height.saturating_sub(safe_height)) as f32 / 2.0;
            self.set_outer_position(PhysicalPosition::new(x, y));
        }
        self
    }
}

fn create_window(event_loop: &ActiveEventLoop, size: PhysicalSize<u32>) -> WindowHandle {
    let attrs = WindowAttributes::default()
        .with_inner_size(size)
        .with_title("App")
        .with_decorations(false)
        .with_resizable(true);

    Arc::new(event_loop.create_window(attrs).expect("Failed to create window"))
        .try_fit_center_to_monitor()
}

fn is_minimized(window: &WindowHandle) -> bool {
    window.is_minimized().unwrap_or(false)
}

fn update_resize_cursor(window: &WindowHandle, position: PhysicalPosition<f64>) {
    #[cfg(not(target_os = "macos"))]
    window.set_cursor(resize_cursor(resize_direction(window, position)));
}

fn resize_direction(
    window: &WindowHandle,
    position: PhysicalPosition<f64>,
) -> Option<ResizeDirection> {
    if window.is_maximized() {
        return None;
    }

    const BORDER: f64 = 6.0;
    let size = window.inner_size();
    let left = position.x <= BORDER;
    let right = position.x >= size.width as f64 - BORDER;
    let top = position.y <= BORDER;
    let bottom = position.y >= size.height as f64 - BORDER;

    match (left, right, top, bottom) {
        (true, false, true, false) => Some(ResizeDirection::NorthWest),
        (false, true, true, false) => Some(ResizeDirection::NorthEast),
        (true, false, false, true) => Some(ResizeDirection::SouthWest),
        (false, true, false, true) => Some(ResizeDirection::SouthEast),
        (true, false, false, false) => Some(ResizeDirection::West),
        (false, true, false, false) => Some(ResizeDirection::East),
        (false, false, true, false) => Some(ResizeDirection::North),
        (false, false, false, true) => Some(ResizeDirection::South),
        _ => None,
    }
}

fn resize_cursor(direction: Option<ResizeDirection>) -> CursorIcon {
    match direction {
        Some(ResizeDirection::North) => CursorIcon::NResize,
        Some(ResizeDirection::South) => CursorIcon::SResize,
        Some(ResizeDirection::West) => CursorIcon::WResize,
        Some(ResizeDirection::East) => CursorIcon::EResize,
        Some(ResizeDirection::NorthWest) => CursorIcon::NwseResize,
        Some(ResizeDirection::NorthEast) => CursorIcon::NeswResize,
        Some(ResizeDirection::SouthWest) => CursorIcon::NeswResize,
        Some(ResizeDirection::SouthEast) => CursorIcon::NwseResize,
        None => CursorIcon::Default,
    }
}

pub(crate) fn set_window_title(window: &WindowHandle, title: &str) {
    window.set_title(title);
}

pub(crate) fn apply_window_action(window: &WindowHandle, action: crate::ui::WindowAction) {
    match action {
        crate::ui::WindowAction::Drag => {
            let _ = window.drag_window();
        }
        crate::ui::WindowAction::Minimize => {
            window.set_minimized(true);
        }
        crate::ui::WindowAction::ToggleMaximize => {
            window.set_maximized(!window.is_maximized());
        }
    }
}

impl<A: RuntimeApp + Default> ApplicationHandler for MyApplication<A> {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        if self.engine.runtime.is_some() {
            return;
        }

        // Window creation is deliberately kept in this bridge. The engine only receives
        // the opaque window handle it needs to initialize its GPU resources.
        let window = create_window(_event_loop, self.size);
        self.engine.resume(window.clone());
        window.request_redraw();
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
        runtime.uilayer.handle_event(&runtime.window, &event);
        if !runtime.uilayer.want_capture_mouse() {
            runtime.input.update_events(&event);
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let Some(runtime) = &self.engine.runtime else {
            return;
        };

        if runtime.wait_for_exit {
            event_loop.exit();
            return;
        }

        if is_minimized(&runtime.window) {
            return;
        }

        runtime.window.request_redraw();
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(runtime) = &mut self.engine.runtime else {
            return;
        };

        let evt = Event::WindowEvent {
            window_id,
            event: event.clone(),
        };

        match &event {
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = Some(*position);
                update_resize_cursor(&runtime.window, *position);
            }
            WindowEvent::CursorLeft { .. } => {
                self.cursor_position = None;
                runtime.window.set_cursor(CursorIcon::Default);
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                #[cfg(not(target_os = "macos"))]
                if let Some(position) = self.cursor_position {
                    if let Some(direction) = resize_direction(&runtime.window, position) {
                        let _ = runtime.window.drag_resize_window(direction);
                    }
                }
            }
            _ => {}
        }

        runtime.uilayer.handle_event(&runtime.window, &evt);
        if !runtime.uilayer.want_capture_mouse() {
            runtime.input.update_events(&evt);
        }

        match event {
            WindowEvent::CloseRequested => {
                self.engine.bus.send_runtime(RuntimeEvent::CloseRequested)
            }
            WindowEvent::Resized(size) => self.engine.bus.send_runtime(RuntimeEvent::Resize {
                width: size.width,
                height: size.height,
            }),
            WindowEvent::RedrawRequested => {
                if !is_minimized(&runtime.window) {
                    self.engine.tick(false);
                }
            }
            WindowEvent::DroppedFile(path) => self
                .engine
                .bus
                .send_runtime(RuntimeEvent::DroppedFile(path)),
            _ => (),
        }
    }
}
