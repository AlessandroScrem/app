use crate::math::*;

#[repr(C)]
#[derive(Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshVertexData {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tangent: [f32; 4],
    pub uv: [f32; 4],
}

impl MeshVertexData {
    const ATTRIBS: [wgpu::VertexAttribute; 4] =
        wgpu::vertex_attr_array![0 =>Float32x3, 1 => Float32x3, 2 => Float32x4, 3 => Float32x4];

    pub fn get_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LinesVertexData {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl LinesVertexData {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 =>Float32x3, 1 => Float32x3];

    pub fn get_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

use crate::uniform::Mat3Std140;

#[repr(C)]
#[derive(Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexInstace {
    pub model: [[f32; 4]; 4],
    pub normal_matrix: [[f32; 4]; 3],
    pub entity_id_low: u32,
    pub entity_id_high: u32,
}


impl VertexInstace {
    const ATTRIBS: [wgpu::VertexAttribute; 9] = wgpu::vertex_attr_array![
        // model matrix (4 vec4)
         5 =>Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Float32x4,
        // normal matrix (3 vec4)
         9 =>Float32x4, 10 => Float32x4, 11 => Float32x4,
        // entity id (1 u64)
        12 => Uint32, 13 => Uint32,
    ];
    
    pub fn get_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

impl VertexInstace {
    pub fn new(model: Mat4, entity_id: u64) -> Self {
        Self {
            model: model.into(),
            normal_matrix: Mat3Std140::inverse_transpose_mat4(&model).into(),
            entity_id_low: (entity_id & 0xFFFFFFFF) as u32,
            entity_id_high: (entity_id >> 32) as u32,
        }
    }
}
