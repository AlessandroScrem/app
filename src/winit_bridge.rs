use std::sync::Arc;

use imgui::{Context, Ui};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{DeviceEvent, ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{CursorIcon, Window, WindowAttributes, WindowId},
};

#[cfg(any(target_os = "windows", target_os = "linux"))]
use winit::window::ResizeDirection;

use crate::app::RuntimeApp;
use crate::engine::{Engine, RuntimeEvent};

pub(crate) type WindowHandle = Arc<Window>;
pub(crate) type WinitEvent = Event<()>;

pub(crate) struct WinitUiPlatform {
    platform: WinitPlatform,
}

impl WinitUiPlatform {
    pub(crate) fn new(context: &mut Context, window: &WindowHandle) -> Self {
        let mut platform = WinitPlatform::new(context);
        platform.attach_window(context.io_mut(), window, HiDpiMode::Default);
        Self { platform }
    }

    pub(crate) fn handle_event(
        &mut self,
        context: &mut Context,
        window: &WindowHandle,
        event: &WinitEvent,
    ) {
        self.platform
            .handle_event(context.io_mut(), window, event);
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
    #[cfg(target_os = "macos")]
    macos_resize: macos_resize::ResizeSession,
}

impl<A: RuntimeApp + Default> MyApplication<A> {
    pub(crate) fn new_with_size(width: u32, height: u32) -> Self {
        Self {
            size: PhysicalSize::new(width, height),
            cursor_position: None,
            ..Default::default()
        }
    }

    pub(crate) fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(&mut self)?;
        Ok(())
    }
}

pub(crate) trait CenterWindow {
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

#[cfg(target_os = "macos")]
mod macos_resize {
    use super::{PhysicalPosition, PhysicalSize, WindowHandle};

    const BORDER: f64 = 6.0;
    const MIN_WIDTH: f64 = 320.0;
    const MIN_HEIGHT: f64 = 200.0;

    #[derive(Clone, Copy)]
    enum Edge {
        North,
        South,
        West,
        East,
        NorthWest,
        NorthEast,
        SouthWest,
        SouthEast,
    }

    #[derive(Clone, Copy)]
    struct ActiveResize {
        edge: Edge,
        cursor: PhysicalPosition<f64>,
        outer: PhysicalPosition<i32>,
        size: PhysicalSize<u32>,
    }

    #[derive(Default)]
    pub(crate) struct ResizeSession {
        active: Option<ActiveResize>,
    }

    impl ResizeSession {
        pub(crate) fn begin(
            &mut self,
            window: &WindowHandle,
            cursor: PhysicalPosition<f64>,
        ) {
            if window.is_maximized() {
                return;
            }

            let Some(edge) = edge_for(window, cursor) else {
                return;
            };
            let Ok(outer) = window.outer_position() else {
                return;
            };

            let absolute_cursor = PhysicalPosition::new(
                outer.x as f64 + cursor.x,
                outer.y as f64 + cursor.y,
            );

            self.active = Some(ActiveResize {
                edge,
                cursor: absolute_cursor,
                outer,
                size: window.inner_size(),
            });
        }

        pub(crate) fn update(
            &mut self,
            window: &WindowHandle,
            cursor: PhysicalPosition<f64>,
        ) {
            let Some(active) = self.active else {
                return;
            };
            let Ok(outer) = window.outer_position() else {
                return;
            };

            let absolute_cursor = PhysicalPosition::new(
                outer.x as f64 + cursor.x,
                outer.y as f64 + cursor.y,
            );
            let dx = absolute_cursor.x - active.cursor.x;
            let dy = absolute_cursor.y - active.cursor.y;

            let mut left = active.outer.x as f64;
            let mut top = active.outer.y as f64;
            let mut width = active.size.width as f64;
            let mut height = active.size.height as f64;

            match active.edge {
                Edge::West | Edge::NorthWest | Edge::SouthWest => {
                    let next_width = (active.size.width as f64 - dx).max(MIN_WIDTH);
                    left += active.size.width as f64 - next_width;
                    width = next_width;
                }
                Edge::East | Edge::NorthEast | Edge::SouthEast => {
                    width = (active.size.width as f64 + dx).max(MIN_WIDTH);
                }
                _ => {}
            }

            match active.edge {
                Edge::North | Edge::NorthWest | Edge::NorthEast => {
                    let next_height = (active.size.height as f64 - dy).max(MIN_HEIGHT);
                    top += active.size.height as f64 - next_height;
                    height = next_height;
                }
                Edge::South | Edge::SouthWest | Edge::SouthEast => {
                    height = (active.size.height as f64 + dy).max(MIN_HEIGHT);
                }
                _ => {}
            }

            window.set_outer_position(PhysicalPosition::new(
                left.round() as i32,
                top.round() as i32,
            ));
            let _ = window.request_inner_size(PhysicalSize::new(
                width.round() as u32,
                height.round() as u32,
            ));
        }

        pub(crate) fn end(&mut self) {
            self.active = None;
        }

        pub(crate) fn edge(
            &self,
            window: &WindowHandle,
            cursor: PhysicalPosition<f64>,
        ) -> Option<Edge> {
            if self.active.is_some() {
                return self.active.map(|active| active.edge);
            }
            edge_for(window, cursor)
        }
    }

    fn edge_for(
        window: &WindowHandle,
        position: PhysicalPosition<f64>,
    ) -> Option<Edge> {
        if window.is_maximized() {
            return None;
        }

        let size = window.inner_size();
        let left = position.x <= BORDER;
        let right = position.x >= size.width as f64 - BORDER;
        let top = position.y <= BORDER;
        let bottom = position.y >= size.height as f64 - BORDER;

        match (left, right, top, bottom) {
            (true, false, true, false) => Some(Edge::NorthWest),
            (false, true, true, false) => Some(Edge::NorthEast),
            (true, false, false, true) => Some(Edge::SouthWest),
            (false, true, false, true) => Some(Edge::SouthEast),
            (true, false, false, false) => Some(Edge::West),
            (false, true, false, false) => Some(Edge::East),
            (false, false, true, false) => Some(Edge::North),
            (false, false, false, true) => Some(Edge::South),
            _ => None,
        }
    }

    pub(crate) fn cursor_icon(
        session: &ResizeSession,
        window: &WindowHandle,
        position: PhysicalPosition<f64>,
    ) -> Option<super::CursorIcon> {
        Some(match session.edge(window, position)? {
            Edge::North => super::CursorIcon::NResize,
            Edge::South => super::CursorIcon::SResize,
            Edge::West => super::CursorIcon::WResize,
            Edge::East => super::CursorIcon::EResize,
            Edge::NorthWest => super::CursorIcon::NwseResize,
            Edge::NorthEast => super::CursorIcon::NeswResize,
            Edge::SouthWest => super::CursorIcon::NeswResize,
            Edge::SouthEast => super::CursorIcon::NwseResize,
        })
    }
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
fn update_resize_cursor(window: &WindowHandle, position: PhysicalPosition<f64>) {
    if let Some(direction) = resize_direction(window, position) {
        window.set_cursor(resize_cursor(Some(direction)));
    }
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
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

#[cfg(any(target_os = "windows", target_os = "linux"))]
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
#[cfg(target_os = "macos")]
fn update_macos_resize_cursor(
    session: &macos_resize::ResizeSession,
    window: &WindowHandle,
    position: PhysicalPosition<f64>,
) {
    if let Some(cursor) = macos_resize::cursor_icon(session, window, position) {
        window.set_cursor(cursor);
    }
}


#[cfg(target_os = "windows")]
mod windows_snap_layout {
    use std::ffi::c_void;

    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    use super::WindowHandle;

    type Hwnd = *mut c_void;
    type LResult = isize;

    const WM_NCHITTEST: u32 = 0x0084;
    const WM_NCDESTROY: u32 = 0x0082;
    const HTMAXBUTTON: LResult = 9;
    const SUBCLASS_ID: usize = 1;
    const TITLE_BAR_HEIGHT: f64 = 36.0;
    const BUTTON_WIDTH: f64 = 36.0;

    #[repr(C)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[repr(C)]
    struct Rect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

    type SubclassProc = unsafe extern "system" fn(
        hwnd: Hwnd,
        msg: u32,
        wparam: usize,
        lparam: isize,
        id: usize,
        ref_data: usize,
    ) -> LResult;

    #[link(name = "comctl32")]
    unsafe extern "system" {
        fn SetWindowSubclass(
            hwnd: Hwnd,
            proc: SubclassProc,
            id: usize,
            ref_data: usize,
        ) -> i32;
        fn RemoveWindowSubclass(hwnd: Hwnd, proc: SubclassProc, id: usize) -> i32;
        fn DefSubclassProc(
            hwnd: Hwnd,
            msg: u32,
            wparam: usize,
            lparam: isize,
        ) -> LResult;
    }

    #[link(name = "user32")]
    unsafe extern "system" {
        fn ScreenToClient(hwnd: Hwnd, point: *mut Point) -> i32;
        fn GetClientRect(hwnd: Hwnd, rect: *mut Rect) -> i32;
        fn GetCursorPos(point: *mut Point) -> i32;
        fn GetDpiForWindow(hwnd: Hwnd) -> u32;
    }

    pub(crate) fn install(window: &WindowHandle) {
        let Ok(handle) = window.window_handle() else {
            return;
        };
        let RawWindowHandle::Win32(handle) = handle.as_raw() else {
            return;
        };

        let hwnd = handle.hwnd.get() as Hwnd;
        unsafe {
            let _ = SetWindowSubclass(hwnd, wnd_proc, SUBCLASS_ID, 0);
        }
    }

    unsafe extern "system" fn wnd_proc(
        hwnd: Hwnd,
        msg: u32,
        wparam: usize,
        lparam: isize,
        _id: usize,
        _ref_data: usize,
    ) -> LResult {
        if msg == WM_NCHITTEST && in_maximize_button(hwnd, lparam) {
            return HTMAXBUTTON;
        }

        let result = unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };

        if msg == WM_NCDESTROY {
            unsafe {
                let _ = RemoveWindowSubclass(hwnd, wnd_proc, SUBCLASS_ID);
            }
        }

        result
    }

    pub(crate) fn maximize_button_hovered(window: &WindowHandle) -> bool {
        let Ok(handle) = window.window_handle() else {
            return false;
        };
        let RawWindowHandle::Win32(handle) = handle.as_raw() else {
            return false;
        };

        let hwnd = handle.hwnd.get() as Hwnd;
        let mut point = Point { x: 0, y: 0 };
        if unsafe { GetCursorPos(&mut point) } == 0 {
            return false;
        }
        if unsafe { ScreenToClient(hwnd, &mut point) } == 0 {
            return false;
        }

        let mut rect = Rect {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        if unsafe { GetClientRect(hwnd, &mut rect) } == 0 {
            return false;
        }

        let dpi = unsafe { GetDpiForWindow(hwnd) }.max(96) as f64;
        let scale = dpi / 96.0;
        let button = BUTTON_WIDTH * scale;
        let titlebar = TITLE_BAR_HEIGHT * scale;

        (point.x as f64) >= rect.right as f64 - button * 2.0
            && (point.x as f64) < rect.right as f64 - button
            && (point.y as f64) >= 0.0
            && (point.y as f64) < titlebar
    }

    fn in_maximize_button(hwnd: Hwnd, lparam: isize) -> bool {
        let mut point = Point {
            x: (lparam as i32 & 0xFFFF) as i16 as i32,
            y: ((lparam as i32 >> 16) & 0xFFFF) as i16 as i32,
        };
        if unsafe { ScreenToClient(hwnd, &mut point) } == 0 {
            return false;
        }

        let mut rect = Rect {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        if unsafe { GetClientRect(hwnd, &mut rect) } == 0 {
            return false;
        }

        let dpi = unsafe { GetDpiForWindow(hwnd) }.max(96) as f64;
        let scale = dpi / 96.0;
        let button = BUTTON_WIDTH * scale;
        let titlebar = TITLE_BAR_HEIGHT * scale;

        point.x as f64 >= rect.right as f64 - button * 2.0
            && (point.x as f64) < rect.right as f64 - button
            && point.y as f64 >= 0.0
            && (point.y as f64) < titlebar
    }
}

pub(crate) fn maximize_button_hovered(window: &WindowHandle) -> bool {
    #[cfg(target_os = "windows")]
    {
        return windows_snap_layout::maximize_button_hovered(window);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = window;
        false
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

        #[cfg(target_os = "windows")]
        windows_snap_layout::install(&window);

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
                #[cfg(any(target_os = "windows", target_os = "linux"))]
                update_resize_cursor(&runtime.window, *position);
                #[cfg(target_os = "macos")]
                {
                    self.macos_resize.update(&runtime.window, *position);
                    update_macos_resize_cursor(&self.macos_resize, &runtime.window, *position);
                }
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
                if let Some(position) = self.cursor_position {
                    #[cfg(any(target_os = "windows", target_os = "linux"))]
                    if let Some(direction) = resize_direction(&runtime.window, position) {
                        let _ = runtime.window.drag_resize_window(direction);
                    }

                    #[cfg(target_os = "macos")]
                    self.macos_resize.begin(&runtime.window, position);
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                #[cfg(target_os = "macos")]
                self.macos_resize.end();
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
                let minimized = self
                    .engine
                    .runtime
                    .as_ref()
                    .is_some_and(|runtime| is_minimized(&runtime.window));
                if !minimized {
                    self.engine.tick(false);
                }
            }
            WindowEvent::DroppedFile(path) => self
                .engine
                .bus
                .send_runtime(RuntimeEvent::DroppedFile(path)),
            _ => {}
        }
    }
}
