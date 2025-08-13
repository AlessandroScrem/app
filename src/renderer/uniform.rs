use cgmath::{Matrix, Matrix4, SquareMatrix};

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

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelUniform {
    pub model: [[f32; 4]; 4],
    pub normal_matrix: [[f32; 4]; 4],
}

impl Default for ModelUniform {
    fn default() -> Self {
        Self {
            model: Matrix4::<f32>::identity().into(),
            normal_matrix: Matrix4::<f32>::identity().into(),
        }
    }
}

impl ModelUniform {
    pub fn new(model: Matrix4<f32>) -> Self {
        let normal_matrix = model
            .invert()
            .unwrap_or(Matrix4::identity())
            .transpose();

        Self {
            model: model.into(),
            normal_matrix: normal_matrix.into(),
        }
    }
}
