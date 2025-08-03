use std::{collections::HashMap, sync::Mutex};

use crate::renderer::uniform::CameraUniform;
use wgpu::util::DeviceExt;

pub struct GPUResourceManager {
    pub camera_uniform_buffer: wgpu::Buffer,

    pub bind_group_layouts: Mutex<HashMap<String, wgpu::BindGroupLayout>>,
    pub bind_groups: Mutex<HashMap<String, wgpu::BindGroup>>,
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

        let mut  layouts = HashMap::new();
        layouts.insert("camera".into(), camera_bind_group_layout);

        let mut groups = HashMap::new();
        groups.insert("camera".into(), camera_bind_group);



        Self {
            camera_uniform_buffer,
            bind_groups: Mutex::new(groups),
            bind_group_layouts: Mutex::new(layouts),
        }
    }

    pub fn add_bind_group_layout(&self, name: &str, bind_group_layout: wgpu::BindGroupLayout) {
        let mut map = self.bind_group_layouts.lock().unwrap();
        map.insert(name.into(), bind_group_layout);
    }

    pub fn add_bind_group(&self, name: &str, bind_group: wgpu::BindGroup) {
        let mut map = self.bind_groups.lock().unwrap();
        map.insert(name.to_string(), bind_group);
    }
}
