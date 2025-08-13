use std::{collections::HashMap, sync::{Arc, Mutex}};

use crate::renderer::uniform::{CameraUniform, ModelUniform};
use wgpu::util::DeviceExt;

pub struct GPUResourceManager {
    pub camera_uniform_buffer: wgpu::Buffer,
    pub model_uniform_buffer: wgpu::Buffer,

    pub bind_group_layouts: Mutex<HashMap<String, Arc<wgpu::BindGroupLayout>>>,
    pub bind_groups: Mutex<HashMap<String, wgpu::BindGroup>>,
}

impl GPUResourceManager {
    pub fn new(device: &wgpu::Device) -> Self {
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

        let camera_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[CameraUniform::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_uniform_buffer.as_entire_binding(),
            }],
            label: Some("Camera Bind Group"),
        });

        let texture_bind_group_layout =
            device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        // sampler
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                        // main
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        // normal map
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        // roughness map
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                    ],
                    label: Some("Texture_bind_group_layout"),
                });

        let model_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Model Bind Group Layout"),
            });

        let model_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Model Uniform Buffer"),
            contents: bytemuck::cast_slice(&[ModelUniform::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let model_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &model_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: model_uniform_buffer.as_entire_binding(),
            }],
            label: Some("Model Bind Group"),
        });

        let mut layouts = HashMap::new();
        layouts.insert("camera".into(), Arc::new(camera_bind_group_layout));
        layouts.insert("texture".into(), Arc::new(texture_bind_group_layout));
        layouts.insert("model".into(), Arc::new(model_bind_group_layout));

        let mut groups = HashMap::new();
        groups.insert("camera".into(), camera_bind_group);
        groups.insert("model".into(), model_bind_group);

        Self {
            camera_uniform_buffer,
            model_uniform_buffer,
            bind_groups: Mutex::new(groups),
            bind_group_layouts: Mutex::new(layouts),
        }
    }

    pub fn get_layout(&self, name: &str) -> Option<Arc<wgpu::BindGroupLayout>> {
        let map = self.bind_group_layouts.lock().unwrap();
        match map.get(name) {
            Some(layout) => Some(layout.clone()),
            None => None,
        }
    }
}
