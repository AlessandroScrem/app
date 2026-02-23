use std::collections::HashSet;
use winit::{event::Event, keyboard::Key};

use crate::math::{Vec2, Zero};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) enum MouseButton {
    Left,
    Middle,
    Right,
    X1,
    X2,
}

fn map_mouse_button(button: winit::event::MouseButton) -> Option<MouseButton> {
    match button {
        winit::event::MouseButton::Left => Some(MouseButton::Left),
        winit::event::MouseButton::Right => Some(MouseButton::Right),
        winit::event::MouseButton::Middle => Some(MouseButton::Middle),
        winit::event::MouseButton::Other(8) => Some(MouseButton::X1),
        winit::event::MouseButton::Other(9) => Some(MouseButton::X2),
        _ => None,
    }
}

#[derive(Debug)]
pub(crate) struct Input {
    keys_down: HashSet<Key>,
    keys_pressed: HashSet<Key>,
    keys_released: HashSet<Key>,

    mouse_buttons_down: HashSet<MouseButton>,
    mouse_buttons_pressed: HashSet<MouseButton>,
    mouse_buttons_released: HashSet<MouseButton>,
    cursor_moved: bool,
    pub(crate) mouse_position: Vec2,
    pub(crate) mouse_delta: Vec2,
    pub(crate) mouse_wheel_movement: Option<Vec2>,
}

impl Input {
    pub(crate) fn new() -> Self {
        Self {
            keys_down: HashSet::new(),
            keys_pressed: HashSet::new(),
            keys_released: HashSet::new(),

            mouse_buttons_down: HashSet::new(),
            mouse_buttons_pressed: HashSet::new(),
            mouse_buttons_released: HashSet::new(),
            mouse_position: Vec2::zero(),
            mouse_delta: Vec2::zero(),
            mouse_wheel_movement: None,
            cursor_moved: false,
        }
    }

    pub(crate) fn is_key_down(&self, key: Key) -> bool {
        self.keys_down.contains(&key)
    }

    pub(crate) fn is_key_pressed(&self, key: Key) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub(crate) fn is_key_released(&self, key: Key) -> bool {
        self.keys_released.contains(&key)
    }

    pub(crate) fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        self.mouse_buttons_down.contains(&button)
    }

    pub(crate) fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_pressed.contains(&button)
    }

    pub(crate) fn is_mouse_button_released(&self, button: MouseButton) -> bool {
        self.mouse_buttons_released.contains(&button)
    }

    pub(crate) fn is_cursor_moved(&self) -> bool {
        self.cursor_moved
    }

    pub(crate) fn update_events<T>(&mut self, event: &Event<T>) {
        match event {
            Event::WindowEvent { event,  ..} => {
                self.update_window_events(event);
            }
            Event::DeviceEvent { event, .. } => {
                self.update_device_events(event);
            }
            _ => (),
        }
    }

    fn update_window_events(&mut self, winit_event: &winit::event::WindowEvent) {
        match winit_event {
            winit::event::WindowEvent::KeyboardInput { event, .. } => {
                let key = event.logical_key.clone();
                if event.state.is_pressed() {
                    self.keys_down.insert(key.clone());
                    self.keys_pressed.insert(key.clone());
                } else {
                    self.keys_down.remove(&key.clone());
                    self.keys_released.insert(key.clone());
                }
            }
            winit::event::WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
                ..
            } => {
                if let Some(mouse_button) = map_mouse_button(*button) {
                    if *state == winit::event::ElementState::Pressed {
                        self.mouse_buttons_down.insert(mouse_button);
                        self.mouse_buttons_pressed.insert(mouse_button);
                    } else if *state == winit::event::ElementState::Released {
                        self.mouse_buttons_down.remove(&mouse_button);
                        self.mouse_buttons_released.insert(mouse_button);
                    }
                }
            }
            winit::event::WindowEvent::MouseWheel { delta, .. } => {
                self.mouse_wheel_movement = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => Some(Vec2::new(*x, *y)),
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        Some(Vec2::new(pos.x as f32, pos.y as f32))
                    }
                };
            }
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = Vec2::new(position.x as f32, position.y as f32);
                self.cursor_moved = true;
            }
            _ => (),
        }
    }

    fn update_device_events(&mut self, winit_event: &winit::event::DeviceEvent) {
        match winit_event {
            winit::event::DeviceEvent::MouseMotion { delta } => {
                self.mouse_delta = Vec2::new(delta.0 as f32, delta.1 as f32);
            }
            _ => (),
        }
    }

    pub(crate) fn clear(&mut self) {
        self.keys_pressed.clear();
        self.keys_released.clear();
        self.mouse_buttons_pressed.clear();
        self.mouse_buttons_released.clear();
        self.mouse_delta = Vec2::zero();
        self.mouse_wheel_movement = None;
        self.cursor_moved = false;
    }
}
