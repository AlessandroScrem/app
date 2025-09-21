use crate::GlobalUniform;
use crate::renderer::uniform::CameraUniform;
use wgpu::BindGroupLayout;
use wgpu::util::DeviceExt;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum LayoutKind {
    Camera,
    Globals,
    Lines,
    Light,
    LightTexture,
    LightIbl,
    Material,
    Model,
    Skybox,
    Hdr,
    Equirect,
}

pub struct GPUResourceManager {
    pub globals_uniform_buffer: wgpu::Buffer,
    pub camera_uniform_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub globals_bind_group: wgpu::BindGroup,
    pub axis_vertexbuffer: wgpu::Buffer,
    layouts: Vec<BindGroupLayout>,
}

impl GPUResourceManager {
    pub fn new(device: &wgpu::Device) -> Self {
        let layouts: Vec<BindGroupLayout> = LayoutKind::iter()
            .map(|kind| create_layout(device, kind))
            .collect();

        let camera_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[CameraUniform::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let globals_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Globals Uniform Buffer"),
            contents: bytemuck::cast_slice(&[GlobalUniform::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layouts[LayoutKind::Camera as usize],
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_uniform_buffer.as_entire_binding(),
            }],
            label: Some("Camera Bind Group"),
        });

        let globals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layouts[LayoutKind::Globals as usize],
            entries: &[
                // Camera
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_uniform_buffer.as_entire_binding(),
                },
                // GLobals
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: globals_uniform_buffer.as_entire_binding(),
                },
            ],
            label: Some("Globals Bind Group"),
        });

        let axis_vertexbuffer = create_axis_buffer(device);

        Self {
            camera_uniform_buffer,
            globals_uniform_buffer,
            camera_bind_group,
            globals_bind_group,
            axis_vertexbuffer,
            layouts,
        }
    }

    pub fn get_layout(&self, kind: LayoutKind) -> &wgpu::BindGroupLayout {
        &self.layouts[kind as usize]
    }
}

fn create_layout(device: &wgpu::Device, kind: LayoutKind) -> BindGroupLayout {
    match kind {
        LayoutKind::Camera => device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
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
        }),
        LayoutKind::Globals => device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Globals Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    // Camera
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    // Globls
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        }),
        LayoutKind::Lines => device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Lines Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                // Camera
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        }),
        LayoutKind::Light => device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Light Bind Group Layout"),
            entries: &[
                // light uniform buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        }),
        LayoutKind::LightTexture => {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Light Texture_bind_group_layout"),
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
                ],
            })
        }
        LayoutKind::LightIbl => {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Light Ibl_bind_group_layout"),
                entries: &[
                    // light uniform buffer
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // irradiance texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::Cube,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    // prefiltered texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::Cube,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    // brdf_lut texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                ],
            })
        }
        LayoutKind::Material => device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Material_bind_group_layout"),
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
                // metallic roughness map
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
                // material Uniform buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        }),
        LayoutKind::Model => device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Model Bind Group Layout"),
            entries: &[
                //model
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        }),
        LayoutKind::Skybox => device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Skybox bind_group_layout"),
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
                        view_dimension: wgpu::TextureViewDimension::Cube,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
            ],
        }),
        LayoutKind::Hdr => device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Hdr_Texture_bind_group_layout"),
            entries: &[
                // sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // hdr texture
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
            ],
        }),
        LayoutKind::Equirect => device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Equirectangular_Texture_bind_group_layout"),
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
            ],
        }),
    }
}

#[repr(C)]
#[derive(Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LinesVertexData {
    position: [f32; 3],
    color: [f32; 3],
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

fn create_axis_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    #[rustfmt::skip] let vertices = [
        LinesVertexData{color: [1.0,0.0,0.0], ..Default::default()},
        LinesVertexData{position: [1.0, 0.0, 0.0], color: [1.0, 0.0, 0.0]}, // X axis red
        LinesVertexData{color: [0.0,1.0,0.0], ..Default::default()},
        LinesVertexData{position: [0.0, 1.0, 0.0], color: [0.0, 1.0, 0.0]}, // y axis green
        LinesVertexData{color: [0.0,0.0,1.0], ..Default::default()},
        LinesVertexData{position: [0.0, 0.0, 1.0], color: [0.0, 0.0, 1.0]}, // z axis blue
    ];

    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Lines Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    })
}
