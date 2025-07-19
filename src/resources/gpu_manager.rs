use crate::renderer::{
    gpu_renderer::{VERTICES, Vertex, VertexBuffer},
    uniform::CameraUniform,
};

use std::{collections::HashMap, sync::Arc};
use wgpu::{RenderPipeline, util::DeviceExt};

pub struct GPUResourceManager {
    _bind_group_layouts: HashMap<String, Arc<wgpu::BindGroupLayout>>,

    pub camera_bind_group: wgpu::BindGroup,
    pub camera_uniform_buffer: wgpu::Buffer,
    pub vertex_buffer: VertexBuffer,

    pub render_pipeline: RenderPipeline,
}

impl GPUResourceManager {
    pub fn new(device: &wgpu::Device, surface_config: &wgpu::SurfaceConfiguration) -> Self {
        let mut bind_group_layouts = HashMap::new();

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

        let shader = device.create_shader_module(wgpu::include_wgsl!("../shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = crate::renderer::pipeline::create_pipeline(
            &device,
            &render_pipeline_layout,
            &surface_config,
            shader,
            Vertex::desc(),
        );

        bind_group_layouts.insert("camera".to_string(), Arc::new(camera_bind_group_layout));

        Self {
            _bind_group_layouts: bind_group_layouts,
            camera_bind_group,
            camera_uniform_buffer,
            vertex_buffer,
            render_pipeline,
        }
    }
}
