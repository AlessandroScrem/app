use cgmath::SquareMatrix;

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
}

impl Default for ModelUniform {
    fn default() -> Self {
        Self {
            model: cgmath::Matrix4::<f32>::identity().into(),
        }
    }
}
