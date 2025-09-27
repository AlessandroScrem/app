// passi per creare un environment IBL
// CreateBRDFintegrationMap();
// CubemapFromHDR(skybox);
// CreateIrradiance(skybox);
// CreatePrefilterMap(skybox);

#![allow(dead_code)]

use std::path::PathBuf;

use crate::{
    assets::texture_manager::TextureManager,
    renderer::{
        gpu_manager::{GPUResourceManager, LayoutKind},
        pipeline_manager,
    },
};
use wgpu::{TextureFormat, TextureViewDescriptor, util::DeviceExt};

use crate::assets::texture;

mod utils {
    use crate::renderer::uniform;
    use wgpu::util::DeviceExt;

    /// Calculate the size of a mip level based on the original size and the mip level index.
    /// # Arguments
    /// * `source` - The original size of the texture (width or height).
    /// * `mip_level` - The mip level index (0 for the original size, 1 for the first mip level, etc.).
    /// # Returns
    /// * The size of the texture at the specified mip level.
    pub fn mip_size(source: u32, mip_level: u32) -> u32 {
        (((source as f32) * 0.5f32.powf(mip_level as f32)).floor() as u32).max(1)
    }

    /// Calculate the number of mip levels for a texture of a given size.
    /// # Arguments
    /// * `texture_size` - The size of the texture (width or height).
    /// # Returns
    /// * The number of mip levels.
    pub fn mip_levels(texture_size: u32) -> u32 {
        (1.0 + (texture_size as f32).log2().floor()) as u32
    }

    /// Create a texture view for a specific face and mip level of a cubemap texture.
    /// # Arguments
    /// * `texture` - The cubemap texture.
    /// * `face_index` - The index of the face (0-5 for a cubemap).
    /// * `mip_level` - The mip level to create the view for.
    /// # Returns
    /// * A `wgpu::TextureView` for the specified face and mip level.
    pub fn create_dest_view(
        texture: &wgpu::Texture,
        face_index: u32,
        mip_level: u32,
    ) -> wgpu::TextureView {
        texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(wgpu::TextureFormat::Rgba16Float), // stesso formato della texture
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: mip_level, // mip level da usare
            mip_level_count: Some(1),  // solo 1 livello
            base_array_layer: face_index,
            array_layer_count: Some(1), // solo il primo layer
            ..Default::default()
        })
    }

    pub fn render_to_cubemap(
        encoder: &mut wgpu::CommandEncoder,
        pipeline: &wgpu::RenderPipeline,
        prefilter_bind_group: &wgpu::BindGroup,
        frame_view: &wgpu::TextureView,
        capture_size: u32,
    ) {
        let clear_color = wgpu::Color::BLACK;

        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: frame_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        renderpass.set_viewport(0.0, 0.0, capture_size as f32, capture_size as f32, 0.0, 1.0);

        renderpass.set_pipeline(pipeline);
        renderpass.set_bind_group(0, prefilter_bind_group, &[]);
        renderpass.draw(0..36, 0..1);
    }

    /// Create camera views for each face of a cubemap.
    /// # Returns
    /// * A vector of 6 `cgmath::Matrix4<f32>` representing the view matrices for each cubemap face.
    pub fn create_camera_views() -> Vec<cgmath::Matrix4<f32>> {
        use cgmath::{Matrix4, Point3};

        const ZERO: Point3<f32> = Point3::new(0.0, 0.0, 0.0);
        const PX: [f32; 3] = [1.0, 0.0, 0.0];
        const NX: [f32; 3] = [-1.0, 0.0, 0.0];
        const PY: [f32; 3] = [0.0, 1.0, 0.0];
        const NY: [f32; 3] = [0.0, -1.0, 0.0];
        const PZ: [f32; 3] = [0.0, 0.0, 1.0];
        const NZ: [f32; 3] = [0.0, 0.0, -1.0];

        vec![
            // +X (right)
            Matrix4::look_at_lh(ZERO, PX.into(), NY.into()),
            // -X (left)
            Matrix4::look_at_lh(ZERO, NX.into(), NY.into()),
            // +Y (top)
            Matrix4::look_at_lh(ZERO, PY.into(), PZ.into()),
            // -Y (bottom)
            Matrix4::look_at_lh(ZERO, NY.into(), NZ.into()),
            // +Z (front)
            Matrix4::look_at_lh(ZERO, PZ.into(), NY.into()),
            // -Z (back)
            Matrix4::look_at_lh(ZERO, NZ.into(), NY.into()),
        ]
    }

    /// Create a camera uniform buffer.
    /// # Arguments
    /// * `device` - The wgpu device to create the buffer.
    /// # Returns
    /// * A `wgpu::Buffer` representing the camera uniform buffer.
    pub fn create_camera_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniform::CameraUniform::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    /// Update the camera uniform buffer with the provided view matrix.
    /// # Arguments
    /// * `queue` - The wgpu queue to write the buffer.
    /// * `camera_uniform_buffer` - The camera uniform buffer to update.
    /// * `cam_view` - The view matrix to set in the camera uniform.
    /// # Returns
    /// * None.
    pub fn update_camera_buffer(
        queue: &wgpu::Queue,
        camera_uniform_buffer: &wgpu::Buffer,
        cam_view: cgmath::Matrix4<f32>,
    ) {
        let cam_proj = cgmath::perspective(cgmath::Deg::<f32>(90.0), 1.0, 0.1, 10.0);

        let updated_uniforms = uniform::CameraUniform {
            view: cam_view.into(),
            proj: cam_proj.into(),
            ..Default::default()
        };

        queue.write_buffer(
            camera_uniform_buffer,
            0,
            bytemuck::bytes_of(&updated_uniforms),
        );
    }

    pub fn create_cube_texture(
        device: &wgpu::Device,
        label: &str,
        size: u32,
        mip_level_count: u32,
        format: wgpu::TextureFormat,
    ) -> wgpu::Texture {
        let width = size;
        let height = size;
        let depth_or_array_layers = 6; // 6 faces
        create_texture(
            device,
            label,
            width,
            height,
            depth_or_array_layers,
            mip_level_count,
            format,
        )
    }

    pub fn create_texture(
        device: &wgpu::Device,
        label: &str,
        width: u32,
        height: u32,
        depth_or_array_layers: u32,
        mip_level_count: u32,
        format: wgpu::TextureFormat,
    ) -> wgpu::Texture {
        let extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: extent,
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        texture
    }
}

pub struct BRDFLUTBuilder {}

impl BRDFLUTBuilder {
    pub fn build(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
        let format = wgpu::TextureFormat::Rg16Float;
        let pipeline = Self::create_pipeline(device, format);
        let size = 512;
        let mip_level_count = 1;
        let depth_or_array_layers = 1;

        let dest_texture = utils::create_texture(
            device,
            "BRDF_LUT",
            size,
            size,
            depth_or_array_layers,
            mip_level_count,
            format,
        );
        let dest_view = dest_texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&Default::default());
        Self::render_to_texture(&mut encoder, &pipeline, &dest_view);

        queue.submit([encoder.finish()]);

        dest_texture
    }

    fn render_to_texture(
        encoder: &mut wgpu::CommandEncoder,
        pipeline: &wgpu::RenderPipeline,
        frame_view: &wgpu::TextureView,
    ) {
        let clear_color = wgpu::Color::BLACK;

        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: frame_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        renderpass.set_pipeline(pipeline);
        renderpass.draw(0..6, 0..1);
    }

    fn create_pipeline(device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("BRDFLUT Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/brdflut.wgsl"));
        let buffer_desc = &[];

        let pipeline_desc = pipeline_manager::PipelineDesc {
            depth_stencil: None,
            blend: None,
            ..Default::default()
        };

        let pipeline = pipeline_desc.build_pipeline(
            "BRDFLUT Pipeline",
            &device,
            render_pipeline_layout,
            format,
            shader,
            buffer_desc,
        );

        pipeline
    }
}

struct PrefilerMapResources {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,
    roughness_buffer: wgpu::Buffer,
}

impl PrefilerMapResources {
    fn new(
        device: &wgpu::Device,
        hdr_view: &wgpu::TextureView,
        format: wgpu::TextureFormat,
    ) -> Self {
        let layout = Self::create_bind_group_layout(device);
        let camera_buffer = utils::create_camera_buffer(device);
        let roughness_buffer = Self::create_roughness_buffer(device);
        let bind_group =
            Self::create_bind_group(device, hdr_view, &camera_buffer, &roughness_buffer, &layout);
        let pipeline = Self::create_pipeline(device, &layout, format);

        Self {
            pipeline,
            bind_group,
            camera_buffer,
            roughness_buffer,
        }
    }

    fn create_pipeline(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[layout],
                push_constant_ranges: &[],
            });

        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/prefilter_map.wgsl"));
        let buffer_desc = &[];

        let pipeline_desc = pipeline_manager::PipelineDesc {
            depth_stencil: None,
            ..Default::default()
        };

        let pipeline = pipeline_desc.build_pipeline(
            "Prefilter Pipeline",
            &device,
            render_pipeline_layout,
            format,
            shader,
            buffer_desc,
        );

        pipeline
    }

    fn create_roughness_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Roughness Uniform Buffer"),
            contents: bytemuck::cast_slice(&[f32::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Prefiler_bind_group_layout"),
            entries: &[
                // sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // cube view
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
                // camera uniform
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // roughness uniform
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
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

    fn create_bind_group(
        device: &wgpu::Device,
        view: &wgpu::TextureView,
        camera_buffer: &wgpu::Buffer,
        roughness_buffer: &wgpu::Buffer,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("PrefilterMap_bind_group"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: roughness_buffer.as_entire_binding(),
                },
            ],
        })
    }
}

pub struct PrefilterMap {}

impl PrefilterMap {
    fn build(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        cube_texture: &wgpu::Texture,
    ) -> wgpu::Texture {
        const TARGET_SIZE: u32 = 128;
        let mip_level_count = 5; // cap to 5 mip levels
        let format = wgpu::TextureFormat::Rgba16Float;

        let cube_texture_view = cube_texture.create_view(&TextureViewDescriptor {
            label: Some("Cube texture view"),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });

        let target_texture = utils::create_cube_texture(
            device,
            "Prefilter_map",
            TARGET_SIZE,
            mip_level_count,
            format,
        );

        render_cubemap_with_resources::<PrefilerMapResources>(
            device,
            queue,
            &cube_texture_view,
            &target_texture,
            TARGET_SIZE,
            mip_level_count,
            format,
        );

        target_texture
    }
}

struct EquirectResources {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,
}

impl EquirectResources {
    fn new(
        device: &wgpu::Device,
        hdr_view: &wgpu::TextureView,
        format: wgpu::TextureFormat,
    ) -> Self {
        let layout = Self::create_bind_group_layout(device);
        let camera_buffer = utils::create_camera_buffer(device);
        let bind_group = Self::create_bind_group(device, hdr_view, &camera_buffer, &layout);
        let pipeline = Self::create_pipeline(device, &layout, format);

        Self {
            pipeline,
            bind_group,
            camera_buffer,
        }
    }

    fn create_pipeline(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[layout],
                push_constant_ranges: &[],
            });

        let shader = device.create_shader_module(wgpu::include_wgsl!(
            "shaders/equirectangular_to_cubemap.wgsl"
        ));
        let buffer_desc = &[];

        let pipeline_desc = pipeline_manager::PipelineDesc {
            depth_stencil: None,
            ..Default::default()
        };

        let pipeline = pipeline_desc.build_pipeline(
            "Equirect Pipeline",
            &device,
            render_pipeline_layout,
            format,
            shader,
            buffer_desc,
        );

        pipeline
    }

    fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Equirectangular_bind_group_layout"),
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
                wgpu::BindGroupLayoutEntry {
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

    fn create_bind_group(
        device: &wgpu::Device,
        view: &wgpu::TextureView,
        unifomrm_buffer: &wgpu::Buffer,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: unifomrm_buffer.as_entire_binding(),
                },
            ],
            label: Some("equirect_bind_group"),
        })
    }
}

pub struct EquirectangularToCubemap {}

impl EquirectangularToCubemap {
    pub fn build(
        hdr_texture: &texture::Texture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        cube_size: u32,
    ) -> wgpu::Texture {
        let dest_format = wgpu::TextureFormat::Rgba16Float;
        let mip_level_count = utils::mip_levels(cube_size);

        // create dest: HDR cubemap texture
        let target_texture = utils::create_cube_texture(
            &device,
            "Hdr_Cube",
            cube_size,
            mip_level_count,
            dest_format,
        );

        render_cubemap_with_resources::<EquirectResources>(
            device,
            queue,
            &hdr_texture.view,
            &target_texture,
            cube_size,
            mip_level_count,
            dest_format,
        );

        target_texture
    }
}

struct IrradianceResources {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,
}

impl IrradianceResources {
    fn new(
        device: &wgpu::Device,
        hdr_view: &wgpu::TextureView,
        format: wgpu::TextureFormat,
    ) -> Self {
        let layout = Self::create_bind_group_layout(device);
        let camera_buffer = utils::create_camera_buffer(device);
        let bind_group = Self::create_bind_group(device, hdr_view, &camera_buffer, &layout);
        let pipeline = Self::create_pipeline(device, &layout, format);

        Self {
            pipeline,
            bind_group,
            camera_buffer,
        }
    }

    fn create_pipeline(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[layout],
                push_constant_ranges: &[],
            });

        let shader =
            device.create_shader_module(wgpu::include_wgsl!("shaders/irradiance_convolution.wgsl"));
        let buffer_desc = &[];

        let pipeline_desc = pipeline_manager::PipelineDesc {
            depth_stencil: None,
            ..Default::default()
        };

        let pipeline = pipeline_desc.build_pipeline(
            "Irradiance Pipeline",
            &device,
            render_pipeline_layout,
            format,
            shader,
            buffer_desc,
        );

        pipeline
    }

    fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Irradiance_bind_group_layout"),
            entries: &[
                // sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // cube view
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
                // camera uniform
                wgpu::BindGroupLayoutEntry {
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

    fn create_bind_group(
        device: &wgpu::Device,
        view: &wgpu::TextureView,
        unifomrm_buffer: &wgpu::Buffer,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: unifomrm_buffer.as_entire_binding(),
                },
            ],
            label: Some("Irradiance_bind_group"),
        })
    }
}

pub struct IrrarianceMap {}

impl IrrarianceMap {
    pub fn build(
        cube_texture: &wgpu::Texture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> wgpu::Texture {
        let cube_size: u32 = 32;
        let dest_format = wgpu::TextureFormat::Rgba16Float;
        let mip_level_count = 1;

        let cube_texture_view = cube_texture.create_view(&TextureViewDescriptor {
            label: Some("Cube texture view"),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });

        let target_texture = utils::create_cube_texture(
            &device,
            "Irradiance_Cube",
            cube_size,
            mip_level_count,
            dest_format,
        );

        render_cubemap_with_resources::<IrradianceResources>(
            device,
            queue,
            &cube_texture_view,
            &target_texture,
            cube_size,
            mip_level_count,
            dest_format,
        );

        target_texture
    }
}

trait CubemapBuilderResources {
    fn new(
        device: &wgpu::Device,
        src_view: &wgpu::TextureView,
        format: wgpu::TextureFormat,
    ) -> Self;
    fn pipeline(&self) -> &wgpu::RenderPipeline;
    fn bind_group(&self) -> &wgpu::BindGroup;
    fn camera_buffer(&self) -> &wgpu::Buffer;
    // Optional: for resources that need per-mip updates (like roughness)
    fn update_per_mip(&self, _queue: &wgpu::Queue, _mip_level: u32, _mip_level_count: u32) {
        // Default: do nothing
    }
}

impl CubemapBuilderResources for IrradianceResources {
    fn new(
        device: &wgpu::Device,
        src_view: &wgpu::TextureView,
        format: wgpu::TextureFormat,
    ) -> Self {
        IrradianceResources::new(device, src_view, format)
    }
    fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
    fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
    fn camera_buffer(&self) -> &wgpu::Buffer {
        &self.camera_buffer
    }
}

impl CubemapBuilderResources for EquirectResources {
    fn new(
        device: &wgpu::Device,
        src_view: &wgpu::TextureView,
        format: wgpu::TextureFormat,
    ) -> Self {
        EquirectResources::new(device, src_view, format)
    }
    fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
    fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
    fn camera_buffer(&self) -> &wgpu::Buffer {
        &self.camera_buffer
    }
}

impl CubemapBuilderResources for PrefilerMapResources {
    fn new(
        device: &wgpu::Device,
        src_view: &wgpu::TextureView,
        format: wgpu::TextureFormat,
    ) -> Self {
        PrefilerMapResources::new(device, src_view, format)
    }
    fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
    fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
    fn camera_buffer(&self) -> &wgpu::Buffer {
        &self.camera_buffer
    }
    fn update_per_mip(&self, queue: &wgpu::Queue, mip_level: u32, mip_level_count: u32) {
        // Update roughness buffer for each mip level
        let roughness = if mip_level_count > 1 {
            mip_level as f32 / (mip_level_count - 1) as f32
        } else {
            0.0
        };
        queue.write_buffer(&self.roughness_buffer, 0, bytemuck::bytes_of(&roughness));
    }
}

// Generic function to render a cubemap using a resource type
fn render_cubemap_with_resources<R: CubemapBuilderResources>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    src_view: &wgpu::TextureView,
    target_texture: &wgpu::Texture,
    cube_size: u32,
    mip_level_count: u32,
    format: wgpu::TextureFormat,
) {
    let resources = R::new(device, src_view, format);
    let camera_views = utils::create_camera_views();

    for mip_level in 0..mip_level_count {
        let capture_size = utils::mip_size(cube_size, mip_level);

        resources.update_per_mip(queue, mip_level, mip_level_count);

        camera_views.iter().enumerate().for_each(|(i, view)| {
            let mut encoder = device.create_command_encoder(&Default::default());
            utils::update_camera_buffer(queue, resources.camera_buffer(), *view);

            let dest_view = utils::create_dest_view(target_texture, i as u32, mip_level);

            utils::render_to_cubemap(
                &mut encoder,
                resources.pipeline(),
                resources.bind_group(),
                &dest_view,
                capture_size,
            );
            queue.submit([encoder.finish()]);
        });
    }
}

pub struct Skybox {
    hdr_path: std::path::PathBuf,
    _cube_map: wgpu::Texture,
    _cube_map_view: wgpu::TextureView,
    _irradiance_map: wgpu::Texture,
    _prefilter_map: wgpu::Texture,
    irradiance_view: wgpu::TextureView,
    prefilter_view: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
}

pub struct SkyboxManager {
    brdf_lut: wgpu::Texture,
    brdf_lut_view: wgpu::TextureView,
    skybox: Skybox,
}

impl SkyboxManager {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        gpu_resource_manager: &GPUResourceManager,
        texture_manager: &mut TextureManager,
    ) -> Self {
        // Create BRDF LUT texture for PBR
        let brdf_lut = BRDFLUTBuilder::build(device, queue);
        let brdf_lut_view = brdf_lut.create_view(&wgpu::TextureViewDescriptor::default());

        #[rustfmt::skip] let hdr_path = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/core/clarens_night_02_2k_16bit.hdr"));
        // Create skybox
        let skybox = Self::create_skybox(
            device,
            queue,
            gpu_resource_manager,
            texture_manager,
            hdr_path,
        );

        Self {
            brdf_lut,
            brdf_lut_view,
            skybox,
        }
    }

    pub fn get_skybox(&self) -> &wgpu::BindGroup {
        &self.skybox.bind_group
    }
    pub fn get_hdr_path(&self) -> &PathBuf {
        &self.skybox.hdr_path
    }

    pub fn get_irradiance(&self) -> &wgpu::TextureView {
        &self.skybox.irradiance_view
    }
    pub fn get_prefilter(&self) -> &wgpu::TextureView {
        &self.skybox.prefilter_view
    }
    pub fn get_brdf_lut(&self) -> &wgpu::TextureView {
        &self.brdf_lut_view
    }

    pub fn change_skybox(
        &mut self,
        hdr_path: &std::path::Path,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        gpu_resource_manager: &GPUResourceManager,
        texture_manager: &mut TextureManager,
    ) {
        if !hdr_path.exists() {
            return;
        }

        self.skybox = Self::create_skybox(
            device,
            queue,
            gpu_resource_manager,
            texture_manager,
            hdr_path,
        );
    }

    fn create_skybox(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        gpu_resource_manager: &GPUResourceManager,
        texture_manager: &mut TextureManager,
        hdr_path: &std::path::Path,
    ) -> Skybox {
        let hdr = texture_manager.get_or_create(hdr_path, TextureFormat::Rgba16Float);
        let cube_map = EquirectangularToCubemap::build(&hdr, device, queue, 512);
        let _irradiance_map = IrrarianceMap::build(&cube_map, device, queue);
        let _prefilter_map = PrefilterMap::build(device, queue, &cube_map);
        let cube_map_view = cube_map.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let bind_group_layout = gpu_resource_manager.get_layout(LayoutKind::Skybox);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&cube_map_view),
                },
            ],
            label: Some("skybox_bind_group"),
        });

        let irradiance_view = _irradiance_map.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });
        let prefilter_view = _prefilter_map.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });

        Skybox {
            hdr_path: hdr_path.into(),
            _cube_map: cube_map,
            _cube_map_view: cube_map_view,
            _irradiance_map,
            irradiance_view,
            _prefilter_map,
            prefilter_view,
            bind_group,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::assets::texture_manager::TextureManager;
    use std::path::Path;

    /// BRDFLut
    #[test]
    fn should_create_brdflut_rg16f_texture() {
        let (device, queue) = crate::get_device_and_queue();

        let brdflut = BRDFLUTBuilder::build(device, queue);

        assert_eq!(brdflut.format(), wgpu::TextureFormat::Rg16Float);
        assert_eq!(brdflut.width(), 512);
        assert_eq!(brdflut.height(), 512);
        assert_eq!(brdflut.mip_level_count(), 1); // <- only 1 mip level
        assert_eq!(brdflut.depth_or_array_layers(), 1); // <- 2D texture
        assert_eq!(brdflut.dimension(), wgpu::TextureDimension::D2);

        #[cfg(feature = "save_tests")]
        {
            use crate::test_utils;
            test_utils::save_texture(&device, &queue, "brdflut.png", &brdflut, 0).unwrap();
        }
    }

    /// PrefilterMap
    #[test]
    fn should_create_prefilter_rgba16f_cubemap() {
        let (device, queue) = crate::get_device_and_queue();
        let mut texture_manager = TextureManager::new(device.clone(), queue.clone());

        #[rustfmt::skip] let filepath = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/test/clarens_night_02_256.hdr"));
        let hdr_texture = texture_manager.get_or_create(filepath, wgpu::TextureFormat::Rgba16Float);
        let cubemap = EquirectangularToCubemap::build(&hdr_texture, &device, &queue, 512);

        let prefilter = PrefilterMap::build(device, queue, &cubemap);

        assert_eq!(prefilter.format(), wgpu::TextureFormat::Rgba16Float);
        assert_eq!(prefilter.height(), 128);
        assert_eq!(prefilter.width(), 128);
        assert_eq!(prefilter.mip_level_count(), 5); // cup to 5 mip levels 
        assert_eq!(prefilter.depth_or_array_layers(), 6); // <- cubemap
        assert_eq!(prefilter.dimension(), wgpu::TextureDimension::D2);

        #[cfg(feature = "save_tests")]{
            use crate::test_utils;
            test_utils::save_cubemap_cross(&device, &queue, "prefilter.png", &prefilter).unwrap();
        }
    }

    /// EquirectangularToCubemap
    #[test]
    fn should_crate_cubetexture_rgba16f_from_equirectangular() {
        let (device, queue) = crate::get_device_and_queue();
        let mut texture_manager = TextureManager::new(device.clone(), queue.clone());

        #[rustfmt::skip] let filepath = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/test/clarens_night_02_256.hdr"));
        let hdr_texture = texture_manager.get_or_create(filepath, wgpu::TextureFormat::Rgba16Float);

        let cubemap = EquirectangularToCubemap::build(&hdr_texture, &device, &queue, 512);

        assert_eq!(cubemap.format(), wgpu::TextureFormat::Rgba16Float);
        assert_eq!(cubemap.height(), 512);
        assert_eq!(cubemap.width(), 512);
        assert_eq!(cubemap.mip_level_count(), 10); // <- log2(512) + 1
        assert_eq!(cubemap.depth_or_array_layers(), 6); // <- cubemap
        assert_eq!(cubemap.dimension(), wgpu::TextureDimension::D2);

        // +X right, -X left, +Y top, -Y bottom, +Z front, -Z back
        #[cfg(feature = "save_tests")] {
            use crate::test_utils;
            test_utils::save_cubemap_cross(&device, &queue, "cubemap.png", &cubemap).unwrap();
        }
    }

    /// IrradianceCubemap
    #[test]
    fn should_crate_irradiance_cubetexture_rgba16f() {
        let (device, queue) = crate::get_device_and_queue();
        let mut texture_manager = TextureManager::new(device.clone(), queue.clone());

        #[rustfmt::skip] let filepath = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/test/clarens_night_02_256.hdr"));
        let hdr_texture = texture_manager.get_or_create(filepath, wgpu::TextureFormat::Rgba16Float);
        let cubemap = EquirectangularToCubemap::build(&hdr_texture, &device, &queue, 512);

        let irradiance = IrrarianceMap::build(&cubemap, &device, &queue);

        assert_eq!(irradiance.format(), wgpu::TextureFormat::Rgba16Float);
        assert_eq!(irradiance.width(), 32);
        assert_eq!(irradiance.height(), 32);
        assert_eq!(irradiance.mip_level_count(), 1); // <- only 1 mip level
        assert_eq!(irradiance.depth_or_array_layers(), 6); // <- cubemap
        assert_eq!(irradiance.dimension(), wgpu::TextureDimension::D2);

        #[cfg(feature = "save_tests")]{
            use crate::test_utils;
            test_utils::save_cubemap_cross(&device, &queue, "Irradiance.png", &irradiance).unwrap();
        }
    }

    /// Skybox
    #[test]
    fn skybox_manager_is_initialized() {
        let (device, queue) = crate::get_device_and_queue();
        let gpu_resource_manager = GPUResourceManager::new(&device);
        let mut texture_manager = TextureManager::new(device.clone(), queue.clone());

        let _manager =
            SkyboxManager::new(&device, &queue, &gpu_resource_manager, &mut texture_manager);
    }

    #[test]
    fn should_create_skybox_from_6_images() {
        let (device, queue) = crate::get_device_and_queue();
        let mut texture_manager = TextureManager::new(device.clone(), queue.clone());

        #[rustfmt::skip] let f0 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/test/right.png"));
        #[rustfmt::skip] let f1 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/test/left.png"));
        #[rustfmt::skip] let f2 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/test/top.png"));
        #[rustfmt::skip] let f3 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/test/bottom.png"));
        #[rustfmt::skip] let f4 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/test/front.png"));
        #[rustfmt::skip] let f5 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/test/back.png"));

        let _cube = texture_manager.create_cubemap(
            f0,
            f1,
            f2,
            f3,
            f4,
            f5,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        );

        #[cfg(feature = "save_tests")]{
            use crate::test_utils;
            test_utils::save_cubemap_cross(&device, &queue, "Skybox_result.png", &_cube.inner).unwrap();
        }
    }

    /// Utils
    #[test]
    fn should_calculate_mip_size() {
        assert_eq!(utils::mip_size(300, 0), 300);
        assert_eq!(utils::mip_size(300, 1), 150);
        assert_eq!(utils::mip_size(300, 2), 75);
        assert_eq!(utils::mip_size(300, 3), 37);
        assert_eq!(utils::mip_size(300, 4), 18);
        assert_eq!(utils::mip_size(300, 5), 9);
        assert_eq!(utils::mip_size(300, 6), 4);
        assert_eq!(utils::mip_size(300, 7), 2);
        assert_eq!(utils::mip_size(300, 8), 1);
        assert_eq!(utils::mip_size(300, 9), 1);
    }

    #[test]
    fn should_calculate_mip_levels() {
        assert_eq!(utils::mip_levels(300), 9);
        assert_eq!(utils::mip_levels(256), 9);
        assert_eq!(utils::mip_levels(128), 8);
        assert_eq!(utils::mip_levels(64), 7);
        assert_eq!(utils::mip_levels(32), 6);
        assert_eq!(utils::mip_levels(16), 5);
        assert_eq!(utils::mip_levels(8), 4);
        assert_eq!(utils::mip_levels(4), 3);
        assert_eq!(utils::mip_levels(2), 2);
        assert_eq!(utils::mip_levels(1), 1);
    }

    #[test]
    fn should_create_camera_views() {
        let views = utils::create_camera_views();
        assert_eq!(views.len(), 6);
    }
}
