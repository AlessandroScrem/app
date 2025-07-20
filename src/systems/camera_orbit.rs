pub fn camera_orbit_system() -> impl legion::systems::Runnable {
    use legion::IntoQuery;
    use legion::SystemBuilder;
    use legion::Write;
    use crate::input::MouseButton;

    SystemBuilder::new("Camera Orbit")
        .read_resource::<crate::DeltaTime>()
        .read_resource::<crate::input::Input>()
        .with_query(<Write<crate::camera::Camera>>::query())
        .build(|_cmd, world, (_delta_time, input), camera_query | {
            for camera in camera_query.iter_mut(world) {
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
        })
}