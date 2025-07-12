use cgmath::{
    perspective, Deg, EuclideanSpace, InnerSpace, Matrix4, Point3, Quaternion, Rad, Rotation3, SquareMatrix, Vector3
};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);


#[derive(Clone, Debug)]
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
    pub fn default() ->Self {
        const FOV: cgmath::Deg<f32> = cgmath::Deg::<f32>(45.0);
        Camera::new(FOV, 1.0, 0.1, 100.0)
    }

    pub fn new<F: Into<Rad<f32>> + std::marker::Copy>(fov: F, aspect: f32, near: f32, far: f32) -> Self {
        let mut camera = Self {
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
        camera.update_view();
        camera
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
        const GAIN: f32 = 0.008;
        const MAX_DELTA: f32 = 2.4;
        const A: f32 = 0.0366;
        const B: f32 = 0.1778;
        const C: f32 = 0.3021;

        let compute_factor = |value: f32| -> f32 {
            let value = value.min(MAX_DELTA);
            (A * (value * value) - B * value + C) * GAIN
        };

        (compute_factor(1.0), compute_factor(self.aspect.recip()))
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

    pub fn get_fov(&self)-> Deg<f32> {
        self.fov.into()
    }

    pub fn set_fov(&mut self, fov: Deg<f32>) {
        self.fov = fov.into();
    }
}
