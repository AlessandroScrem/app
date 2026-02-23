use cgmath::{Matrix, Matrix3};

use crate::math::*;

///shader: [pbr, blinnphong, equirectangular_to_cubemap, irradiance_convolution, light, lines, prefilter_map, skybox]
#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct CameraUniform {
    pub(crate) view_position: [f32; 4],
    pub(crate) view: [[f32; 4]; 4],
    pub(crate) proj: [[f32; 4]; 4],
    pub(crate) screen_size: [f32; 2],
    pub(crate) _pad: [f32; 2],
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
pub(crate) struct ModelUniform {
    pub(crate) model: [[f32; 4]; 4],
    normal_matrix: Mat4x3,
    pub(crate) entity_id: u64,
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
    pub(crate) fn new(model: Mat4) -> Self {
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
pub(crate) struct GlobalUniform {
    pub(crate) ibl_enable: u32,
    pub(crate) skybox_enable: u32,
    pub(crate) exposure: f32,
    pub(crate) ibl_intensity: f32,
    pub(crate) entity_id: u64,
    pub(crate) tonemap_filter: u32,
    pub(crate) debug: u32,
}

///shader: [pbr, blinnphong, light]
#[repr(C, align(16))]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct LightUniform {
    pub(crate) color: [f32; 3],
    pub(crate) directional: u32,
    pub(crate) position: [f32; 3],
    pub(crate) cast_shadow: u32,
    pub(crate) entity_id: u64,
    pub(crate) pad2: [i32; 2],
}
impl Default for LightUniform {
    fn default() -> Self {
        Self {
            color: [1.0, 1.0, 1.0],
            cast_shadow: 0,
            directional: 1,
            position: [0.0, 0.0, -1.0],
            entity_id: 0,
            pad2: [0, 0],
        }
    }
}

///shader: [pbr, blinnphong]
#[repr(C, align(16))]
#[derive(Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct MaterialUniform {
    pub(crate) color_factor: [f32; 4],
    pub(crate) emissive_factor: [f32; 4],
    pub(crate) roughness_factor: f32,
    pub(crate) metallic_factor: f32,
    pub(crate) normal_scale: f32,
    pub(crate) occlusion_strength: f32,
    pub(crate) use_color_texture: u32,
    pub(crate) use_metal_roughness_texture: u32,
    pub(crate) use_normal_texture: u32,
    pub(crate) use_emissive_texture: u32,
    pub(crate) use_occlusion_texture: u32,
    pub(crate) pad: [u32;3],
}

