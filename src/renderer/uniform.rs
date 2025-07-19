use crate::camera;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_position: [f32; 4],
    pub view_proj: [[f32; 4]; 4],
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self {
            view_position: [0f32; 4],
            view_proj: [[0f32; 4]; 4],
        }
    }
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &camera::Camera/* , projection: &camera::Projection */) {
        // self.view_position = camera.position.to_homogeneous().into();
        // self.view_proj = (projection.calc_matrix() * camera.get_matrix()).into();
        self.view_position = camera.get_position().to_homogeneous().into();
        self.view_proj = (camera.get_projection() * camera.get_matrix()).into();
    }
}