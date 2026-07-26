use std::collections::HashSet;
use winit::{
    event::MouseButton as WinitMouseButton,
    event::{DeviceEvent, Event, WindowEvent},
    keyboard::Key as WinitKey,
    keyboard::NamedKey,
    event::MouseScrollDelta,
};

use crate::math::{Vec2, Zero};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Extra1,
    Extra2,
    X1,
    X2,
}
fn map_mouse_button(button: WinitMouseButton) -> Option<MouseButton> {
    match button {
        WinitMouseButton::Left | WinitMouseButton::Other(0) => Some(MouseButton::Left),
        WinitMouseButton::Right | WinitMouseButton::Other(1) => Some(MouseButton::Right),
        WinitMouseButton::Middle | WinitMouseButton::Other(2) => Some(MouseButton::Middle),
        WinitMouseButton::Other(3) => Some(MouseButton::Extra1),
        WinitMouseButton::Other(4) => Some(MouseButton::Extra2),
        WinitMouseButton::Other(8) => Some(MouseButton::X1),
        WinitMouseButton::Other(9) => Some(MouseButton::X2),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyButton {
    Alt,
}

fn map_keyboard(key: WinitKey) -> Option<KeyButton> {
    match key {
        WinitKey::Named(NamedKey::Alt) => Some(KeyButton::Alt),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct Input {
    keys_down: HashSet<KeyButton>,
    keys_pressed: HashSet<KeyButton>,

    mouse_buttons_down: HashSet<MouseButton>,
    mouse_buttons_pressed: HashSet<MouseButton>,
    cursor_moved: bool,
    pub mouse_position: Vec2,
    pub mouse_delta: Vec2,
    pub mouse_wheel_movement: Option<Vec2>,
}

impl Input {
    pub fn new() -> Self {
        Self {
            keys_down: HashSet::new(),
            keys_pressed: HashSet::new(),

            mouse_buttons_down: HashSet::new(),
            mouse_buttons_pressed: HashSet::new(),
            mouse_position: Vec2::zero(),
            mouse_delta: Vec2::zero(),
            mouse_wheel_movement: None,
            cursor_moved: false,
        }
    }

    pub fn is_key_down(&self, key: KeyButton) -> bool {
        self.keys_down.contains(&key)
    }

    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        self.mouse_buttons_down.contains(&button)
    }

    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_pressed.contains(&button)
    }

    pub fn is_cursor_moved(&self) -> bool {
        self.cursor_moved
    }

    pub fn update_events<T>(&mut self, event: &Event<T>) {
        match event {
            Event::WindowEvent { event, .. } => {
                self.update_window_events(event);
            }
            Event::DeviceEvent { event, .. } => {
                self.update_device_events(event);
            }
            _ => (),
        }
    }

    fn update_window_events(&mut self, winit_event: &WindowEvent) {
        match winit_event {
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(key) = map_keyboard(event.logical_key.clone()) {
                    if event.state.is_pressed() {
                        self.keys_down.insert(key);
                        self.keys_pressed.insert(key);
                    } else {
                        self.keys_down.remove(&key);
                    }
                }
            }
            WindowEvent::MouseInput {
                state,
                button,
                ..
            } => {
                if let Some(mouse_button) = map_mouse_button(*button) {
                    if state.is_pressed() {
                        self.mouse_buttons_down.insert(mouse_button);
                        self.mouse_buttons_pressed.insert(mouse_button);
                    } else {
                        self.mouse_buttons_down.remove(&mouse_button);
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.mouse_wheel_movement = match delta {
                    MouseScrollDelta::LineDelta(x, y) => Some(Vec2::new(*x, *y)),
                    MouseScrollDelta::PixelDelta(pos) => {
                        Some(Vec2::new(pos.x as f32, pos.y as f32))
                    }
                };
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = Vec2::new(position.x as f32, position.y as f32);
                self.cursor_moved = true;
            }
            _ => (),
        }
    }

    fn update_device_events(&mut self, winit_event: &DeviceEvent) {
        match winit_event {
            DeviceEvent::MouseMotion { delta } => {
                self.mouse_delta = Vec2::new(delta.0 as f32, delta.1 as f32);
            }
            _ => (),
        }
    }

    pub fn clear(&mut self) {
        self.keys_pressed.clear();
        self.mouse_buttons_pressed.clear();
        self.mouse_delta = Vec2::zero();
        self.mouse_wheel_movement = None;
        self.cursor_moved = false;
    }
}
