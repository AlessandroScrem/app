use wgpu::BindGroupLayout;
use wgpu::util::DeviceExt;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use super::*;
use crate::assets::{ColorSpace, SamplerDesc};
use crate::assets::vertexdata::LinesVertexData;
use crate::gpu::texture::{GpuTextureBuilder, GpuTextureUsage};
use crate::uniform::{CameraUniform, GlobalUniform, LightUniform};

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
pub enum LayoutKind {
    Camera,
    PerFrame,
    Lines,
    LightTexture,
    Ibl,
    Material,
    Model,
    Skybox,
    Hdr,
    Depth,
    EntityId,
}

struct LayoutCache {
    layouts: Vec<BindGroupLayout>,
}
impl LayoutCache {
    fn new(device: &wgpu::Device) -> Self {
        let layouts: Vec<BindGroupLayout> = LayoutKind::iter()
            .map(|kind| Self::create_layout(device, kind))
            .collect();
        Self { layouts }
    }

    fn get(&self, kind: LayoutKind) -> &wgpu::BindGroupLayout {
        &self.layouts[kind as usize]
    }
}

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum FramebufferKind {
    Hdr,
    EntityId,
    Depth,
}

pub struct Framebuffer {
    texture: GpuTexture,
    bind_group: wgpu::BindGroup,
}

struct FramebufferCache {
    framebuffers: Vec<Framebuffer>,
}
impl FramebufferCache {
    fn new(device: &wgpu::Device, layouts: &LayoutCache, width: u32, height: u32) -> Self {
        let framebuffers: Vec<Framebuffer> = FramebufferKind::iter()
            .map(|kind| Self::create(device, layouts, kind, width, height))
            .collect();
        Self { framebuffers }
    }

    fn resize(&mut self, device: &wgpu::Device, layouts: &LayoutCache, width: u32, height: u32) {
        let framebuffers: Vec<Framebuffer> = FramebufferKind::iter()
            .map(|kind| Self::create(device, layouts, kind, width, height))
            .collect();
        self.framebuffers = framebuffers;
    }

    fn get_texture(&self, kind: FramebufferKind) -> &wgpu::Texture {
        &self.framebuffers[kind as usize].texture.inner
    }
    fn get_view(&self, kind: FramebufferKind) -> &wgpu::TextureView {
        &self.framebuffers[kind as usize].texture.view
    }
    fn get_bg(&self, kind: FramebufferKind) -> &wgpu::BindGroup {
        &self.framebuffers[kind as usize].bind_group
    }
}

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum BindgroupKind {
    Camera,
    Perframe,
    LightTexture,
}

struct BindgroupCache {
    bg: Vec<wgpu::BindGroup>,
}

impl BindgroupCache {
    fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        buffer_cache: &BufferCache,
        layouts: &LayoutCache,
    ) -> Self {
        let bg: Vec<wgpu::BindGroup> = BindgroupKind::iter()
            .map(|kind| Self::create(device, queue, buffer_cache, layouts, kind))
            .collect();
        Self { bg }
    }
    fn get(&self, kind: BindgroupKind) -> &wgpu::BindGroup {
        &self.bg[kind as usize]
    }
}

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum BufferKind {
    Camera,
    Globals,
    Light,
    Axis,
}

struct BufferCache {
    buffers: Vec<wgpu::Buffer>,
}

impl BufferCache {
    fn new(device: &wgpu::Device) -> Self {
        let buffer: Vec<wgpu::Buffer> = BufferKind::iter()
            .map(|kind| Self::create(device, kind))
            .collect();
        Self { buffers: buffer }
    }

    fn get(&self, kind: BufferKind) -> &wgpu::Buffer {
        &self.buffers[kind as usize]
    }
}

pub struct GpuManager {
    layout_cache: LayoutCache,
    framebuffer_cache: FramebufferCache,
    buffer_cache: BufferCache,
    bindgroup_cache: BindgroupCache,
}

impl GpuManager {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32) -> Self {
        let layout_cache = LayoutCache::new(device);
        let buffer_cache = BufferCache::new(device);
        let framebuffer_cache = FramebufferCache::new(device, &layout_cache, width, height);
        let bindgroup_cache = BindgroupCache::new(device, queue, &buffer_cache, &layout_cache);

        Self {
            layout_cache,
            framebuffer_cache,
            buffer_cache,
            bindgroup_cache,
        }
    }

    pub fn get_layout(&self, kind: LayoutKind) -> &wgpu::BindGroupLayout {
        self.layout_cache.get(kind)
    }

    pub fn get_framebuffer_view(&self, kind: FramebufferKind) -> &wgpu::TextureView {
        self.framebuffer_cache.get_view(kind)
    }

    pub fn get_framebuffer_texture(&self, kind: FramebufferKind) -> &wgpu::Texture {
        self.framebuffer_cache.get_texture(kind)
    }
    pub fn get_framebuffer_bg(&self, kind: FramebufferKind) -> &wgpu::BindGroup {
        self.framebuffer_cache.get_bg(kind)
    }

    pub fn resize_frame(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.framebuffer_cache
            .resize(device, &self.layout_cache, width, height);
    }

    pub fn get_bindgroup(&self, kind: BindgroupKind) -> &wgpu::BindGroup {
        self.bindgroup_cache.get(kind)
    }
    pub fn get_buffer(&self, kind: BufferKind) -> &wgpu::Buffer {
        self.buffer_cache.get(kind)
    }
}

impl LayoutCache {
    fn create_layout(device: &wgpu::Device, kind: LayoutKind) -> BindGroupLayout {
        match kind {
            LayoutKind::Camera => {
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                })
            }
            LayoutKind::PerFrame => {
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                        wgpu::BindGroupLayoutEntry {
                            // Ligth
                            binding: 2,
                            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                })
            }
            LayoutKind::Lines => {
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                })
            }
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
            LayoutKind::Ibl => {
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Light Ibl_bind_group_layout"),
                    entries: &[
                        // sampler
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                        // irradiance texture
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
                        // prefiltered texture
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
                        // brdf_lut texture
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
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
            LayoutKind::Material => {
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                        // emissive map
                        wgpu::BindGroupLayoutEntry {
                            binding: 4,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        // occlusion map
                        wgpu::BindGroupLayoutEntry {
                            binding: 5,
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
                            binding: 6,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                })
            }
            LayoutKind::Model => {
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Model Bind Group Layout"),
                    entries: &[
                        //model
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
                })
            }
            LayoutKind::Skybox => {
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                })
            }
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
            LayoutKind::Depth => {
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Depth_Texture_bind_group_layout"),
                    entries: &[
                        // sampler
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                        // depth texture
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Depth,
                            },
                            count: None,
                        },
                    ],
                })
            }
            LayoutKind::EntityId => {
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("ID_Texture_bind_group_layout"),
                    entries: &[
                        // entity_id sampler
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                            count: None,
                        },
                        // entity_id texture
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Uint,
                            },
                            count: None,
                        },
                    ],
                })
            }
        }
    }
}

impl FramebufferCache {
    fn create(
        device: &wgpu::Device,
        layouts: &LayoutCache,
        kind: FramebufferKind,
        width: u32,
        height: u32,
    ) -> Framebuffer {
        match kind {
            FramebufferKind::Hdr => {
                let layout = layouts.get(LayoutKind::Hdr);

                let texture = GpuTextureBuilder::from_empty(width, height)
                    .format(ColorSpace::Rgbaf16)
                    .usage(GpuTextureUsage::RenderTarget)
                    .sampler(SamplerDesc::Nearest)
                    .label("Hdr texture")
                    .build(device, None);

                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Hdr_bind_group"),
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Sampler(&texture.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&texture.view),
                        },
                    ],
                });

                Framebuffer {
                    texture,
                    bind_group,
                }
            }

            FramebufferKind::EntityId => {
                let layout = layouts.get(LayoutKind::EntityId);
                let texture = GpuTextureBuilder::from_empty(width, height)
                    .format(ColorSpace::Rg32ui)
                    .usage(GpuTextureUsage::EntityId)
                    .sampler(SamplerDesc::Nearest)
                    .label("entity_id_texture")
                    .build(device, None);

                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("entity_id_bind_group"),
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Sampler(&texture.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&texture.view),
                        },
                    ],
                });

                Framebuffer {
                    texture,
                    bind_group,
                }
            }
            FramebufferKind::Depth => {
                let layout = layouts.get(LayoutKind::Depth);

                let texture = GpuTextureBuilder::from_empty(width, height)
                    .format(ColorSpace::Depth32f)
                    .usage(GpuTextureUsage::DepthTarget)
                    .sampler(SamplerDesc::Nearest)
                    .label("depth_texture")
                    .build(device, None);

                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("depth_bind_group"),
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Sampler(&texture.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&texture.view),
                        },
                    ],
                });

                Framebuffer {
                    texture,
                    bind_group,
                }
            }
        }
    }
}
impl BindgroupCache {
    fn create(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        buffer_cache: &BufferCache,
        layouts: &LayoutCache,
        kind: BindgroupKind,
    ) -> wgpu::BindGroup {
        match kind {
            BindgroupKind::Camera => device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: layouts.get(LayoutKind::Camera),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer_cache.get(BufferKind::Camera).as_entire_binding(),
                }],
                label: Some("Camera Bind Group"),
            }),
            BindgroupKind::Perframe => {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: layouts.get(LayoutKind::PerFrame),
                    entries: &[
                        // Camera
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: buffer_cache.get(BufferKind::Camera).as_entire_binding(),
                        },
                        // GLobals
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: buffer_cache.get(BufferKind::Globals).as_entire_binding(),
                        },
                        // Light
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: buffer_cache.get(BufferKind::Light).as_entire_binding(),
                        },
                    ],
                    label: Some("PerFrame Bind Group"),
                })
            }
            BindgroupKind::LightTexture => {
                let texture =
                    GpuTextureBuilder::from_static(&static_textures::LIGHTBULB_STATIC_TEXTURE)
                        .build(device, Some(queue));
                let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                    address_mode_u: wgpu::AddressMode::Repeat,
                    address_mode_v: wgpu::AddressMode::Repeat,
                    address_mode_w: wgpu::AddressMode::Repeat,
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Linear,
                    mipmap_filter: wgpu::FilterMode::Linear,
                    ..Default::default()
                });

                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: layouts.get(LayoutKind::LightTexture),
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Sampler(&sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&texture.view),
                        },
                    ],
                    label: Some("light texture_bind_group"),
                })
            }
        }
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
                contents: bytemuck::cast_slice(&[LightUniform::default()]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn should_contain_static_textures() {
//         let (device, queue) = test_utils::get_device_and_queue();
//         let gpu_mgr = GpuManager::new(&device, &queue, 32, 32);

//         let _texture = gpu_mgr.static_textures.lightbulb;

//         // #[cfg(feature = "save_tests")]
//         test_utils::save_texture(device, queue, "texture.png", &_texture, 0).unwrap()
//     }
// }
