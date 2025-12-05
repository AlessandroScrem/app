use cgmath::{Matrix, Matrix3};

use crate::math::*;

///shader: [pbr, blinnphong, equirectangular_to_cubemap, irradiance_convolution, light, lines, prefilter_map, skybox]
#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_position: [f32; 4],
    pub view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
    pub screen_size: [f32; 2],
    pub _pad: [f32; 2],
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self {
            view_position: [0f32; 4],
            view: [[0f32; 4]; 4],
            proj: [[0f32; 4]; 4],
            screen_size: [1.0f32; 2],
            _pad: [0f32; 2],
        }
    }
}

#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Mat4x3 {
    m: [[f32; 4]; 3],
}

impl Mat4x3 {
    fn mat3_to_std140(m: Matrix3<f32>) -> [[f32; 4]; 3] {
        [
            [m.x.x, m.x.y, m.x.z, 0.0],
            [m.y.x, m.y.y, m.y.z, 0.0],
            [m.z.x, m.z.y, m.z.z, 0.0],
        ]
    }
    fn identity() -> Self {
        Self {
            m: Self::mat3_to_std140(Matrix3::identity()),
        }
    }

    fn inverse_transpose(mat: &Mat4) -> Self {
        let mat3x3 = Matrix3::from_cols(mat.x.truncate(), mat.y.truncate(), mat.z.truncate());
        let nm = mat3x3.invert().unwrap_or(Matrix3::identity()).transpose();

        Self {
            m: Self::mat3_to_std140(nm),
        }
    }
}

///shader: [pbr, blinnphong]
#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelUniform {
    pub model: [[f32; 4]; 4],
    normal_matrix: Mat4x3,
    pub entity_id: u64,
    pad2: [u32; 2],
}

impl Default for ModelUniform {
    fn default() -> Self {
        Self {
            model: Mat4::identity().into(),
            normal_matrix: Mat4x3::identity(),
            entity_id: 0,
            pad2: [0, 0],
        }
    }
}

impl ModelUniform {
    pub fn new(model: Mat4) -> Self {
        Self {
            model: model.into(),
            normal_matrix: Mat4x3::inverse_transpose(&model),
            ..Default::default()
        }
    }
}

///shader: [pbr, hdr]
#[repr(C, align(16))]
#[derive(Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlobalUniform {
    pub ibl_enable: u32,
    pub skybox_enable: u32,
    pub exposure: f32,
    pub tonemap_filter: u32,
    pub entity_id: u64,
    pub pad2: [u32; 2],
}

///shader: [pbr, blinnphong]
#[repr(C, align(16))]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialUniform {
    pub color: [f32; 4],
    pub roughness: f32,
    pub metallic: f32,
    pub roughness_use_texture: u32,
    pub metallic_use_texture: u32,
    pub color_use_texture: u32,
    pub padding: [u32; 3],
}
impl Default for MaterialUniform {
    fn default() -> Self {
        Self {
            color: [1.0, 1.0, 1.0, 1.0],
            roughness: 1.0,
            metallic: 0.0,
            roughness_use_texture: 0,
            metallic_use_texture: 0,
            color_use_texture: 0,
            padding: [0; 3],
        }
    }
}
