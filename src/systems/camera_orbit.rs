use legion::*;
use crate::input::MouseButton;

#[system(for_each)]
pub fn camera_orbit(
    camera: &mut crate::camera::Camera,
    #[resource] input: &crate::input::Input,
) {
    if input.is_mouse_button_down(MouseButton::Left) {
        let delta = (input.mouse_delta.x as f64, input.mouse_delta.y as f64);
        camera.orbit(delta);
    }

    if input.is_mouse_button_down(MouseButton::Middle) {
        let delta = (input.mouse_delta.x as f64, input.mouse_delta.y as f64);
        camera.pan(delta);
    }

    if let Some(delta) = input.mouse_wheel_movement {
        camera.zoom(delta.y);
    }
}
