use crate::renderer::{uniform::CameraUniform,};

use crate::{VERTICES, VertexBuffer};

use wgpu::util::DeviceExt;

pub struct GPUResourceManager {
    pub camera_bind_group_layout:  wgpu::BindGroupLayout,

    pub camera_bind_group: wgpu::BindGroup,
    pub camera_uniform_buffer: wgpu::Buffer,
    pub vertex_buffer: VertexBuffer,

}

impl GPUResourceManager {
    pub fn new(device: &wgpu::Device) -> Self {
        let camera_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[CameraUniform::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Camera Bind Group Layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_uniform_buffer.as_entire_binding(),
            }],
            label: Some("Camera Bind Group"),
        });

        let vertex_buffer = VertexBuffer(device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            },
        ));

        Self {
            camera_bind_group_layout,
            camera_bind_group,
            camera_uniform_buffer,
            vertex_buffer,
        }
    }
}
