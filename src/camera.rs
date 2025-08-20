use cgmath::{
    EuclideanSpace, InnerSpace, Matrix4, Point3, Quaternion, Rad, Rotation3, SquareMatrix, Vector3,
    perspective,
};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[derive(Clone, Debug)]
pub struct Camera {
    position: Vector3<f32>,
    aspect: f32,
    pub fov: Rad<f32>,
    pub near: f32,
    pub far: f32,
    yaw: f32,
    pitch: f32,
    focal_point: Vector3<f32>,
    distance: f32,
    view_matrix: Matrix4<f32>,
}

impl Camera {
    pub fn default() -> Self {
        const FOV: cgmath::Deg<f32> = cgmath::Deg::<f32>(45.0);
        Camera::new(FOV, 1.0, 0.1, 100.0)
    }

    pub fn new<F: Into<Rad<f32>> + std::marker::Copy>(
        fov: F,
        aspect: f32,
        near: f32,
        far: f32,
    ) -> Self {
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

    pub fn update_view(&mut self) {
        self.position = self.calculate_position();
        let orientation = self.get_orientation();
        let translation = Matrix4::from_translation(self.position);
        self.view_matrix = (translation * Matrix4::from(orientation)).invert().unwrap();
    }

    // move camera in screen asis [left/right] [up/bottom]
    pub fn pan(&mut self, delta: (f64, f64)) {
        let dx = delta.0 as f32;
        let dy = delta.1 as f32;
        let (xspeed, yspeed) = self.pan_speed();
        self.focal_point += -self.get_right_direction() * dx * xspeed * self.distance;
        self.focal_point += self.get_up_direction() * dy * yspeed * self.distance;
        self.update_view();
    }

    // Orbit the camera
    pub fn orbit(&mut self, delta: (f64, f64)) {
        const SPEED: f32 = 0.01;
        let dx = delta.0 as f32;
        let dy = delta.1 as f32;
        let yaw_sign = self.get_up_direction().y.signum();

        self.yaw += yaw_sign * dx * SPEED;
        self.pitch += dy * SPEED;
        self.update_view();
    }

    // move camera position [back/forward] to focal_point
    pub fn zoom(&mut self, delta: f32) {
        let distance = self.distance - delta * self.zoom_speed();
        if distance > 0.0 {
            self.distance = distance;
        }
        self.update_view();
    }

    // getters
    pub fn get_view_mat(&self) -> Matrix4<f32> {
        self.view_matrix
    }

    // Ottiene una projection RH con correzione da opengl [-1, 1] a Vulkan(wgpu) Z [0, 1]
    pub fn get_projection_mat(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(self.fov, self.aspect, self.near, self.far)
    }

    pub fn get_position(&self) -> Point3<f32> {
        EuclideanSpace::from_vec(self.position)
    }

    pub fn get_focal_point(&self) -> Point3<f32> {
        EuclideanSpace::from_vec(self.focal_point)
    }

    pub fn get_yaw_pitch(&self) -> (f32, f32) {
        (self.yaw, self.pitch)
    }

    pub fn get_distance(&self) -> f32 {
        self.distance
    }

    // setters
    pub fn set_distance(&mut self, distance: f32) {
        self.distance = distance;
        self.update_view();
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }

    // RH classico (OpenGL)
    // Forward = -Z
    // Right = +X
    // Up = +Y
    // Regola RH: Up × Forward = Right oppure Right × Up = Forward

    // private
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

    fn calculate_position(&self) -> Vector3<f32> {
        self.focal_point - self.get_forward_direction() * self.distance
    }

    fn pan_speed(&self) -> (f32, f32) {
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
}

#[cfg(test)]
mod tests {

    use cgmath::{EuclideanSpace, InnerSpace, Vector3};

    #[test]
    fn test_camera_update_view() {
        use super::Camera;
        
        let mut cam = Camera::default();

        // Aggiorna la view matrix
        cam.update_view();
        let view = cam.get_view_mat();

        // Estrai i vettori della camera dalla view matrix
        // Column-major: view = inv(translation * rotation)
        // right = view x-axis, up = view y-axis, forward = -view z-axis
        let right = Vector3::new(view.x.x, view.y.x, view.z.x).normalize();
        let up = Vector3::new(view.x.y, view.y.y, view.z.y).normalize();
        let forward = -Vector3::new(view.x.z, view.y.z, view.z.z).normalize();

        // Vettori ortogonali
        let eps = 1e-6;
        assert!((right.dot(up)).abs() < eps, "right non ortogonale a up");
        assert!(
            (right.dot(forward)).abs() < eps,
            "right non ortogonale a forward"
        );
        assert!((up.dot(forward)).abs() < eps, "up non ortogonale a forward");

        // Regola mano sinistra: right × up = -forward
        let cross = right.cross(up).normalize();
        assert!(
            (cross + forward).magnitude() < eps,
            "non LH: right * up = {:?}, expected -forward {:?}",
            cross,
            forward
        );

        // Controlla coerenza con posizione/focal_point
        let expected_forward =
            (cam.get_focal_point().to_vec() - cam.get_position().to_vec()).normalize();
        let dot = forward.dot(expected_forward);
        assert!(
            (dot - 1.0).abs() < eps,
            "forward della view_matrix non coincide con focal point: dot = {}",
            dot
        );
    }

    #[test]
    fn test_camera_perspective_is_rh_and_wgpu_compatible() {
        use super::Camera;
        use cgmath::{Deg, vec4};

        let fovy = Deg(45.0);
        let aspect = 1.0;
        let near = 1.0;
        let far = 100.0;

        // Proiezione RH con z[0, 1]
        let camera = Camera::new(fovy, aspect, near, far);

        let proj = camera.get_projection_mat();

        // Near plane: z = -near (perché RH guarda lungo -Z)
        let near_point = vec4(0.0, 0.0, -near, 1.0);
        let near_proj = proj * near_point;
        let z_ndc_near = near_proj.z / near_proj.w;

        // Far plane: z = -far
        let far_point = vec4(0.0, 0.0, -far, 1.0);
        let far_proj = proj * far_point;
        let z_ndc_far = far_proj.z / far_proj.w;

        let eps = 1e-6;
        // In wgpu ci aspettiamo [0,1]
        assert!(
            (z_ndc_near - 0.0).abs() < eps,
            "near -> expected 0, got {}",
            z_ndc_near
        );
        assert!(
            (z_ndc_far - 1.0).abs() < eps,
            "far  -> expected 1, got {}",
            z_ndc_far
        );
    }

    #[test]
    fn test_cgmath_perspective_is_opengl_style_rh() {
        use cgmath::{Deg, Matrix4, perspective, vec4};

        let fovy = Deg(45.0);
        let aspect = 1.0;
        let near = 1.0;
        let far = 100.0;

        // Proiezione di cgmath
        let proj: Matrix4<f32> = perspective(fovy, aspect, near, far);

        // Near plane: z = -near (perché RH guarda lungo -Z)
        let near_point = vec4(0.0, 0.0, -near, 1.0);
        let near_proj = proj * near_point;
        let z_ndc_near = near_proj.z / near_proj.w;

        // Far plane: z = -far
        let far_point = vec4(0.0, 0.0, -far, 1.0);
        let far_proj = proj * far_point;
        let z_ndc_far = far_proj.z / far_proj.w;

        // In OpenGL-style ci aspettiamo [-1,1]
        let eps = 1e-6;
        assert!(
            (z_ndc_near + 1.0).abs() < eps,
            "near plane not mapped to -1, got {}",
            z_ndc_near
        );
        assert!(
            (z_ndc_far - 1.0).abs() < eps,
            "far plane not mapped to 1, got {}",
            z_ndc_far
        );
    }

    #[test]
    fn test_cgmath_perspective_with_correction_is_wgpu_compatible() {
        use super::OPENGL_TO_WGPU_MATRIX;
        use cgmath::{Deg, Matrix4, perspective, vec4};

        let fovy = Deg(45.0);
        let aspect = 1.0;
        let near = 1.0;
        let far = 100.0;

        // Proiezione di cgmath (RH, OpenGL-style [-1,1])
        let cgmath_proj: Matrix4<f32> = perspective(fovy, aspect, near, far);

        // Applico la correzione OpenGL → wgpu ([-1,1] → [0,1])
        let proj = OPENGL_TO_WGPU_MATRIX * cgmath_proj;

        // Near plane: z = -near (perché RH guarda lungo -Z)
        let near_point = vec4(0.0, 0.0, -near, 1.0);
        let near_proj = proj * near_point;
        let z_ndc_near = near_proj.z / near_proj.w;

        // Far plane: z = -far
        let far_point = vec4(0.0, 0.0, -far, 1.0);
        let far_proj = proj * far_point;
        let z_ndc_far = far_proj.z / far_proj.w;

        let eps = 1e-6;
        // In wgpu ci aspettiamo [0,1]
        assert!(
            (z_ndc_near - 0.0).abs() < eps,
            "near -> expected 0, got {}",
            z_ndc_near
        );
        assert!(
            (z_ndc_far - 1.0).abs() < eps,
            "far  -> expected 1, got {}",
            z_ndc_far
        );
    }

    #[test]
    fn custom_perspective_lh_is_wgpu_compatible() {
        use cgmath::{Matrix4, vec4};
        let fovy = std::f32::consts::FRAC_PI_4; // 45°
        let aspect = 1.0;
        let near = 1.0;
        let far = 100.0;

        fn projection_lh_zo(fovy: f32, aspect: f32, near: f32, far: f32) -> Matrix4<f32> {
            let ctan_fov = 1.0f32 / f32::tan(fovy * 0.5);
            let k: f32 = far / (far - near);

            let proj = Matrix4::<f32>::from_cols(
                vec4(ctan_fov / aspect, 0.0, 0.0, 0.0),
                vec4(0.0, ctan_fov, 0.0, 0.0),
                vec4(0.0, 0.0, k, 1.0),
                vec4(0.0, 0.0, -k * near, 0.0),
            );
            proj
        }

        // LH: forward = +Z, clip depth in [0,1]
        let proj = projection_lh_zo(fovy, aspect, near, far);

        // Punto sul near plane
        let near_point = vec4(0.0, 0.0, near, 1.0);
        let near_proj = proj * near_point;
        let z_ndc_near = near_proj.z / near_proj.w;

        // Punto sul far plane
        let far_point = vec4(0.0, 0.0, far, 1.0);
        let far_proj = proj * far_point;
        let z_ndc_far = far_proj.z / far_proj.w;

        let eps = 1e-6;
        // In wgpu ci aspettiamo [0,1]
        assert!(
            (z_ndc_near - 0.0).abs() < eps,
            "near -> expected 0, got {}",
            z_ndc_near
        );
        assert!(
            (z_ndc_far - 1.0).abs() < eps,
            "far  -> expected 1, got {}",
            z_ndc_far
        );
    }

    /*     #[test]
    fn glam_perspective_lh_is_wgpu_compatible() {
        // glam = "0.30.5"
        use glam::{Mat4, Vec4};
        let fovy = std::f32::consts::FRAC_PI_4; // 45°
        let aspect = 1.0;
        let near = 1.0;
        let far = 100.0;

        // LH: forward = +Z, clip depth in [0,1]
        let proj = Mat4::perspective_lh(fovy, aspect, near, far);

        // punti sui piani near/far (z positive in LH)
        let near_clip = proj * Vec4::new(0.0, 0.0, near, 1.0);
        let far_clip = proj * Vec4::new(0.0, 0.0, far, 1.0);

        let z_ndc_near = near_clip.z / near_clip.w;
        let z_ndc_far = far_clip.z / far_clip.w;

        let eps = 1e-6;
        // In wgpu ci aspettiamo [0,1]
        assert!(
            (z_ndc_near - 0.0).abs() < eps,
            "near -> expected 0, got {}",
            z_ndc_near
        );
        assert!(
            (z_ndc_far - 1.0).abs() < eps,
            "far  -> expected 1, got {}",
            z_ndc_far
        );
    } */
}
