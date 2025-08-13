use crate::input::MouseButton;
use legion::*;

#[system]
pub fn camera_orbit(
    #[resource] camera: &mut crate::camera::Camera,
    #[resource] input: &crate::input::Input,
    #[resource] surface_configuration: &wgpu::SurfaceConfiguration,
) {
    let aspect =
        surface_configuration.width.max(1) as f32 / surface_configuration.height.max(1) as f32;
    camera.set_aspect(aspect);

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
