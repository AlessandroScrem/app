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

impl CameraUniform {
    pub fn from_camera_size(camera: &super::Camera, size: (u32, u32)) -> Self {
        let screen_size = [size.0 as f32, size.1 as f32];
        Self {
            view_position: camera.get_position().to_homogeneous().into(),
            view: camera.get_view_mat().into(),
            proj: camera.get_projection_mat().into(),
            screen_size,
            ..Default::default()
        }
    }
}

impl GlobalUniform {
    pub fn from_global_id(globals: &super::Globals, entity_id: u64) -> Self {
        Self {
            ibl_enable: globals.ibl_enable as u32,
            skybox_enable: globals.skybox_enable as u32,
            exposure: globals.exposure,
            ibl_intensity: globals.ibl_intensity,
            tonemap_filter: globals.tonemap_filter,
            entity_id,
            debug: globals.debug_code,
            ..Default::default()
        }
    }
}

#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Mat4x3 {
    m: [[f32; 4]; 3],
}

impl Mat4x3 {
    fn mat3_to_std140(m: Mat3) -> [[f32; 4]; 3] {
        [
            [m.x.x, m.x.y, m.x.z, 0.0],
            [m.y.x, m.y.y, m.y.z, 0.0],
            [m.z.x, m.z.y, m.z.z, 0.0],
        ]
    }
    fn identity() -> Self {
        Self {
            m: Self::mat3_to_std140(Mat3::identity()),
        }
    }

    fn inverse_transpose(mat: &Mat4) -> Self {
        let mat3x3 = Mat3::from_cols(mat.x.truncate(), mat.y.truncate(), mat.z.truncate());
        let nm = mat3x3.invert().unwrap_or(Mat3::identity()).transpose();

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
    pub ibl_intensity: f32,

    pub entity_id: u64,
    pub tonemap_filter: u32,
    pub debug: u32,
}

///shader: [pbr, blinnphong, light]
#[repr(C, align(16))]
#[derive(Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub color: [f32; 3],
    pub directional: u32,
    pub position: [f32; 3],
    pub cast_shadow: u32,
    pub entity_id: u64,
    pub enabled: u32,
    pub pad: [i32; 1],
}

impl LightUniform {
    fn new() -> Self {
        Self {
            color: [1.0, 1.0, 1.0],
            enabled: 1,
            directional: 1,
            position: [0.0, 0.0, -1.0],
            ..Default::default()
        }
    }
}

pub const MAX_LIGHTS: usize = 64;
#[repr(C, align(16))]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightsUniform {
    pub lights: [LightUniform; MAX_LIGHTS],

    pub count: u32,
    pub enabled: u32,
    pub pad: [u32; 2],
}

impl Default for LightsUniform {
    fn default() -> Self {
        Self {
            lights: [LightUniform::new(); MAX_LIGHTS],
            count: 0,
            enabled: 0,
            pad: [0; 2],
        }
    }
}

///shader: [pbr, blinnphong]
#[repr(C, align(16))]
#[derive(Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialUniform {
    pub color_factor: [f32; 4],
    pub emissive_factor: [f32; 4],

    pub roughness_factor: f32,
    pub metallic_factor: f32,
    pub normal_scale: f32,
    pub occlusion_strength: f32,

    pub texture_flags: u32,
    pub alpha_mode: u32,
    pub alpha_cutoff: f32,
    pub transmission_factor: f32,

    pub is_trasmissive: u32,
    pub pad: [u32; 3],
}
