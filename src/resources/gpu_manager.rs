use crate::renderer::uniform::CameraUniform;
use wgpu::util::DeviceExt;

pub struct GPUResourceManager {
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub texture_bind_group_layout: Option<wgpu::BindGroupLayout>,

    pub camera_bind_group: wgpu::BindGroup,
    pub camera_uniform_buffer: wgpu::Buffer,
    pub texture_bind_group: Option<wgpu::BindGroup>,
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

        Self {
            camera_bind_group_layout,
            camera_bind_group,
            camera_uniform_buffer,
            texture_bind_group_layout: None,
            texture_bind_group: None,
        }
    }

    pub fn add_texture_bind_group(
        &mut self,
        bind_group_layout: wgpu::BindGroupLayout,
        bind_group: wgpu::BindGroup,
    ) {
        self.texture_bind_group_layout = Some(bind_group_layout);
        self.texture_bind_group = Some(bind_group);
    }
}
