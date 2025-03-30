use cgmath::{
    EuclideanSpace, InnerSpace, Matrix4, Point3, Quaternion, Rad, Rotation, Rotation3,
    SquareMatrix, Vector3, perspective,
};
use std::{f32::consts::FRAC_PI_2, time::Duration};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseScrollDelta},
    keyboard::KeyCode,
};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);


#[derive(Debug)]
pub struct Camera {
    position: Vector3<f32>,
    aspect: f32,
    fov: Rad<f32>,
    near: f32,
    far: f32,
    yaw: f32,
    pitch: f32,
    focal_point: Vector3<f32>,
    distance: f32,
    view_matrix: Matrix4<f32>,
}

impl Camera {
    pub fn new<F: Into<Rad<f32>> + std::marker::Copy>(fov: F, aspect: f32, near: f32, far: f32) -> Self {
        let mut instance = Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            aspect,
            fov: fov.into(),
            near,
            far,
            yaw: 0.0,
            pitch: 0.0,
            focal_point: Vector3::new(0.0, 0.0, 0.0),
            distance: 5.0,
            view_matrix: Matrix4::identity(),
        };
        instance.update_view();
        instance
    }

    pub fn get_matrix(&self) -> Matrix4<f32> {
        self.view_matrix
    }

    pub fn get_projection(&self) -> Matrix4<f32> {
         OPENGL_TO_WGPU_MATRIX * perspective(self.fov, self.aspect, self.near, self.far)
    }

    pub fn update_view(&mut self) {
        self.position = self.calculate_position();
        let orientation = self.get_orientation();
        let translation = Matrix4::from_translation(self.position);
        self.view_matrix = (translation * Matrix4::from(orientation)).invert().unwrap();
    }

    fn get_forward_direction(&self) -> Vector3<f32> {
        (self.get_orientation() * Vector3::new(0.0, 0.0, -1.0)).normalize()
    }

    fn get_right_direction(&self) -> Vector3<f32> {
        (self.get_orientation() * Vector3::new(1.0, 0.0, 0.0)).normalize()
    }

    fn get_up_direction(&self) -> Vector3<f32> {
        (self.get_orientation() * Vector3::new(0.0, 1.0, 0.0)).normalize()
    }

    fn get_orientation(&self) -> Quaternion<f32> {
        Quaternion::from_angle_y(Rad(-self.yaw)) * Quaternion::from_angle_x(Rad(-self.pitch))
    }

    pub fn get_position(&self) -> Point3<f32> {
        EuclideanSpace::from_vec(self.position)
    }

    fn calculate_position(&self) -> Vector3<f32> {
        self.focal_point - self.get_forward_direction() * self.distance
    }

    pub fn set_aspect(&mut self, aspect: f32){
        self.aspect = aspect;
    }

    pub fn pan_speed(&self) -> (f32, f32) {
        const GAIN: f32 = 0.01;
        const MAX_DELTA: f32 = 2.4;
        const A: f32 = 0.0366;
        const B: f32 = 0.1778;
        const C: f32 = 0.3021;

        let compute_factor = || -> f32 {
            let value = 1.0_f32.min(MAX_DELTA);
            (A * (value * value) - B * value + C) * GAIN
        };

        (compute_factor(), compute_factor() * self.aspect.recip())
    }

    fn zoom_speed(&self) -> f32 {
        let max_speed = self.far - self.near;
        const ZOOM_GAIN: f32 = 0.2;
        let mut distance = self.distance * ZOOM_GAIN;
        distance = distance.max(0.0);
        let speed = distance.min(max_speed);
        speed
    }

    pub fn pan(&mut self, delta: (f64, f64))  {
        let dx = delta.0 as f32;
        let dy = delta.1 as f32;
        let (xspeed, yspeed) = self.pan_speed();
        self.focal_point += -self.get_right_direction() * dx * xspeed * self.distance;
        self.focal_point += self.get_up_direction() * dy * yspeed * self.distance;
        self.update_view();
    }

    // Orbit the camera
    pub fn orbit(&mut self, delta: (f64, f64)) {
        const  SPEED: f32  = 0.01; 
        let dx = delta.0 as f32;
        let dy = delta.1 as f32;
        let yaw_sign = self.get_up_direction().y.signum();

        self.yaw += yaw_sign * dx * SPEED;
        self.pitch += dy * SPEED;
        self.update_view();
    }

    pub fn zoom(&mut self, delta: f32) {
        let distance = self.distance - delta * self.zoom_speed();
        if distance > 0.0 {
            self.distance = distance;
        }
        self.update_view();
    }
}
/*
#[derive(Debug)]
pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
} */
/*
#[derive(Debug)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn process_keyboard(&mut self, key: KeyCode, state: ElementState) -> bool {
        let amount = if state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };
        match key {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.amount_forward = amount;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.amount_backward = amount;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.amount_left = amount;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.amount_right = amount;
                true
            }
            KeyCode::Space => {
                self.amount_up = amount;
                true
            }
            KeyCode::ShiftLeft => {
                self.amount_down = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = -match delta {
            // I'm assuming a line is about 100 pixels
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => *scroll as f32,
        };
    }

    pub fn _update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        camera.position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

        // Move in/out (aka. "zoom")
        // Note: this isn't an actual zoom. The camera's position
        // changes when zooming. I've added this to make it easier
        // to get closer to an object you want to focus on.
        let (pitch_sin, pitch_cos) = camera.pitch.0.sin_cos();
        let scrollward =
            Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        camera.position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;

        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

        // Rotate
        camera.yaw += Rad(self.rotate_horizontal) * self.sensitivity * dt;
        camera.pitch += Rad(-self.rotate_vertical) * self.sensitivity * dt;

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non-cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        // Keep the camera's angle from going too high/low.
        if camera.pitch < -Rad(SAFE_FRAC_PI_2) {
            camera.pitch = -Rad(SAFE_FRAC_PI_2);
        } else if camera.pitch > Rad(SAFE_FRAC_PI_2) {
            camera.pitch = Rad(SAFE_FRAC_PI_2);
        }
    }
}
 */
