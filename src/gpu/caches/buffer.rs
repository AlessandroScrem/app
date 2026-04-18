use crate::uniform::{CameraUniform, GlobalUniform, LightsUniform};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use wgpu::util::DeviceExt;

use crate::assets::vertexdata::LinesVertexData;
const fn axis() -> [LinesVertexData; 6] {
    const RED: [f32; 3] = [1.0, 0.0, 0.0];
    const GREEN: [f32; 3] = [0.0, 1.0, 0.0];
    const BLUE: [f32; 3] = [0.0, 0.0, 1.0];
    #[rustfmt::skip] let vertices = [
        LinesVertexData{position: [0.0, 0.0, 0.0], color: RED},
        LinesVertexData{position: [10.0, 0.0, 0.0], color: RED},   //X  
        LinesVertexData{position: [0.0, 0.0, 0.0], color: GREEN},
        LinesVertexData{position: [0.0, 10.0, 0.0], color: GREEN}, //Y
        LinesVertexData{position: [0.0, 0.0, 0.0], color: BLUE},
        LinesVertexData{position: [0.0, 0.0, 10.0], color: BLUE},  //Z
    ];
    vertices
}


#[derive(Debug, Clone, Copy, EnumIter)]
pub enum BufferKind {
    Camera,
    Globals,
    Light,
    Axis,
}

pub struct BufferCache {
    buffers: Vec<wgpu::Buffer>,
}

impl BufferCache {
    pub fn new(device: &wgpu::Device) -> Self {
        let buffer: Vec<wgpu::Buffer> = BufferKind::iter()
            .map(|kind| Self::create(device, kind))
            .collect();
        Self { buffers: buffer }
    }

    pub fn get(&self, kind: BufferKind) -> &wgpu::Buffer {
        &self.buffers[kind as usize]
    }
}

impl BufferCache {
    fn create(device: &wgpu::Device, kind: BufferKind) -> wgpu::Buffer {
        match kind {
            BufferKind::Camera => device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Uniform Buffer"),
                contents: bytemuck::cast_slice(&[CameraUniform::default()]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
            BufferKind::Globals => device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Globals Uniform Buffer"),
                contents: bytemuck::cast_slice(&[GlobalUniform::default()]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
            BufferKind::Axis => {
                let vertices = axis();
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Lines Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                })
            }
            BufferKind::Light => device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Light Uniform Buffer"),
                contents: bytemuck::cast_slice(&[LightsUniform::default()]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
        }
    }
}


