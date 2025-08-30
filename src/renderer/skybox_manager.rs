// passi per creare un environment IBL
// CreateBRDFintegrationMap();
// CubemapFromHDR(skybox);
// CreateIrradiance(skybox);
// CreatePrefilterMap(skybox);

use crate::renderer::{
    gpu_manager::{GPUResourceManager, LayoutKind},
    pipeline_manager, uniform,
};

use anyhow::Ok;
use wgpu::{util::DeviceExt, wgt::TextureViewDescriptor};

use crate::assets::texture;

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
        let camera_buffer = Self::create_camera_buffer(device);
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
            &device,
            render_pipeline_layout,
            format,
            shader,
            buffer_desc,
        );

        pipeline
    }

    fn create_camera_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniform::CameraUniform::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
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

pub struct BRDFLUTBuilder {}

impl BRDFLUTBuilder {
    pub fn build(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
        let format = wgpu::TextureFormat::Rg16Float;
        let pipeline = Self::create_pipeline(device, format);
        let size = 512;

        let dest_texture = Self::create_dest_brdflut_texture(device, size, size, format);
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

    fn create_dest_brdflut_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> wgpu::Texture {
        let extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("BRDF_LUT"),
            size: extent,
            mip_level_count: 1,
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
            &device,
            render_pipeline_layout,
            format,
            shader,
            buffer_desc,
        );

        pipeline
    }
}

pub struct Hdr {
    hdr_texture: texture::Texture,
}
impl Hdr {
    const CUBE_FACES: usize = 6;

    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        filepath: &std::path::Path,
        format: wgpu::TextureFormat,
    ) -> Self {
        assert_eq!(
            format,
            wgpu::TextureFormat::Rgba16Float,
            "Hdr support only Rgba16Float"
        );

        let buffer = std::fs::read(filepath).unwrap();
        let hdr_texture = crate::assets::texture::Texture::new(device, queue, &buffer, format);

        Self { hdr_texture }
    }

    pub fn to_cubemap(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: u32,
    ) -> wgpu::Texture {
        // CubemapFromHDR(skybox);
        // -- create equirect texture
        // -- create equirect pipeline
        // -- create dest cube texture
        // -- create dest cube 6 textureview
        // -- set 6 camera view matrix each for cube side
        // -- render equirect to cubemap 6faces

        let dest_format = wgpu::TextureFormat::Rgba8Unorm;
        let width = size;
        let height = size;

        // create dest: cubemap texture LDR (TODO: add tonemap)
        let dest_texture = Self::create_dest_cube_texture(&device, width, height, dest_format);
        let cube_dest_views = Self::create_cube_texture_views(&dest_texture);

        // create camera buffer
        // create bindgrouplayout for hdr attachement
        // create equirect pipeline
        let resources = EquirectResources::new(
            device,
            &self.hdr_texture.view,
            wgpu::TextureFormat::Rgba8Unorm,
        );

        // create camera matrix/views
        let camera_views = Self::create_camera_views();

        // render faces
        camera_views.iter().enumerate().for_each(|(i, view)| {
            let mut encoder = device.create_command_encoder(&Default::default());
            Self::update_camera(&queue, &resources.camera_buffer, *view);

            Self::render_to_cubemap(
                &mut encoder,
                &resources.pipeline,
                &resources.bind_group,
                &cube_dest_views[i],
            );
            queue.submit([encoder.finish()]);
        });

        dest_texture
    }

    fn create_dest_cube_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> wgpu::Texture {
        let extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 6, // <- 6 faces,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Cubemap"),
            size: extent,
            mip_level_count: 1,
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

    fn create_cube_texture_views(
        cube_texture: &wgpu::Texture,
    ) -> [wgpu::TextureView; Self::CUBE_FACES] {
        std::array::from_fn(|i| {
            cube_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some(&format!("Cubemap Face {}", i)),
                dimension: Some(wgpu::TextureViewDimension::D2),
                base_array_layer: i as u32,
                base_mip_level: 0,
                mip_level_count: Some(1),
                array_layer_count: Some(1),
                aspect: wgpu::TextureAspect::All,
                format: None,
                ..Default::default()
            })
        })
    }

    fn create_camera_views() -> Vec<cgmath::Matrix4<f32>> {
        use cgmath::{Matrix4, Point3, Vector3};

        let eye = Point3::new(0.0, 0.0, 0.0);

        vec![
            // +X
            Matrix4::look_at_lh(
                eye,
                Point3::new(1.0, 0.0, 0.0),
                Vector3::new(0.0, -1.0, 0.0),
            ),
            // -X
            Matrix4::look_at_lh(
                eye,
                Point3::new(-1.0, 0.0, 0.0),
                Vector3::new(0.0, -1.0, 0.0),
            ),
            // +Y
            Matrix4::look_at_lh(eye, Point3::new(0.0, 1.0, 0.0), Vector3::new(0.0, 0.0, 1.0)),
            // -Y
            Matrix4::look_at_lh(
                eye,
                Point3::new(0.0, -1.0, 0.0),
                Vector3::new(0.0, 0.0, -1.0),
            ),
            // +Z
            Matrix4::look_at_lh(
                eye,
                Point3::new(0.0, 0.0, 1.0),
                Vector3::new(0.0, -1.0, 0.0),
            ),
            // -Z
            Matrix4::look_at_lh(
                eye,
                Point3::new(0.0, 0.0, -1.0),
                Vector3::new(0.0, -1.0, 0.0),
            ),
        ]
    }

    fn update_camera(
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

    fn render_to_cubemap(
        encoder: &mut wgpu::CommandEncoder,
        pipeline: &wgpu::RenderPipeline,
        equirect_bind_group: &wgpu::BindGroup,
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
        renderpass.set_bind_group(0, equirect_bind_group, &[]);
        renderpass.draw(0..36, 0..1);
    }
}

fn save_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    filename: &str,
    texture: &wgpu::Texture,
) -> anyhow::Result<()> {
    let texture_size = texture.width();

    let u32_size = std::mem::size_of::<u32>() as u32;

    let output_buffer_size = (u32_size * texture_size * texture_size) as wgpu::BufferAddress;
    let output_buffer_desc = wgpu::BufferDescriptor {
        size: output_buffer_size,
        usage: wgpu::BufferUsages::COPY_DST
        // this tells wpgu that we want to read this buffer from the cpu
        | wgpu::BufferUsages::MAP_READ,
        label: None,
        mapped_at_creation: false,
    };
    let output_buffer = device.create_buffer(&output_buffer_desc);

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    {
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(u32_size * texture_size),
                    rows_per_image: Some(texture_size),
                },
            },
            texture.size(),
        );

        queue.submit(Some(encoder.finish()));
    }

    // We need to scope the mapping variables so that we can
    // unmap the buffer
    {
        let buffer_slice = output_buffer.slice(..);

        // The mapping process is async, so we'll need to create a channel to get
        // the success flag for our mapping
        let (tx, rx) = std::sync::mpsc::channel();

        // We send the success or failure of our mapping via a callback
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });

        // The callback we submitted to map async will only get called after the
        // device is polled or the queue submitted
        device.poll(wgpu::PollType::Wait)?;

        // We check if the mapping was successful here
        rx.recv()??;

        let data_rg16f = buffer_slice.get_mapped_range();

        let data_rgba8 = rg16float_to_rgba8(&data_rg16f, texture_size, texture_size);

        use image::{ImageBuffer, Rgba};
        let buffer =
            ImageBuffer::<Rgba<u8>, _>::from_raw(texture_size, texture_size, data_rgba8).unwrap();
        buffer.save(filename).unwrap();
    }
    output_buffer.unmap();

    Ok(())
}

fn rg16float_to_rgba8(raw: &[u8], width: u32, height: u32) -> Vec<u8> {
    use half::f16;
    let mut out = Vec::with_capacity((width * height * 4) as usize);

    for i in 0..(width * height) as usize {
        // Ogni pixel = 4 byte = 2 half-float
        let offset = i * 4;
        let r_half = u16::from_le_bytes([raw[offset], raw[offset + 1]]);
        let g_half = u16::from_le_bytes([raw[offset + 2], raw[offset + 3]]);

        let r = f16::from_bits(r_half).to_f32();
        let g = f16::from_bits(g_half).to_f32();

        // Converti in [0,255]
        let r_u8 = (r.clamp(0.0, 1.0) * 255.0).round() as u8;
        let g_u8 = (g.clamp(0.0, 1.0) * 255.0).round() as u8;

        // B=0, A=255 (o come preferisci)
        out.push(r_u8);
        out.push(g_u8);
        out.push(0);
        out.push(255);
    }

    out
}

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum SkyboxKind {
    Default,
}

struct Skybox {
    _texture: wgpu::Texture,
    _view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
}

pub struct SkyboxManager {
    skyboxes: Vec<Skybox>,
}
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

impl SkyboxManager {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        gpu_resource_manager: &GPUResourceManager,
    ) -> Self {
        let skyboxes: Vec<Skybox> = SkyboxKind::iter()
            .map(|kind| create_skybox(device, queue, gpu_resource_manager, kind))
            .collect();

        Self { skyboxes }
    }

    pub fn get_skybox(&self, kind: SkyboxKind) -> &wgpu::BindGroup {
        &self.skyboxes[kind as usize].bind_group
    }
}

fn create_skybox(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    gpu_resource_manager: &GPUResourceManager,
    kind: SkyboxKind,
) -> Skybox {
    match kind {
        SkyboxKind::Default => {
            #[rustfmt::skip] let filepath = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/core/clarens_night_02_2k.hdr"));
            let hdr = Hdr::new(device, queue, filepath, wgpu::TextureFormat::Rgba16Float);
            let _texture = hdr.to_cubemap(device, queue, 1024);
            let _view = _texture.create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::Cube),
                ..Default::default()
            });

            let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
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
                        resource: wgpu::BindingResource::TextureView(&_view),
                    },
                ],
                label: Some("skybox_bind_group"),
            });
            Skybox {
                _texture,
                _view,
                bind_group,
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::assets::texture_manager::TextureManager;
    use std::path::Path;

    #[test]
    fn should_create_cubemap_from_hdr() {
        let (device, queue) = crate::get_device_and_queue();

        #[rustfmt::skip] let filepath = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/core/clarens_night_02_2k.hdr"));
        let hdr = Hdr::new(device, queue, filepath, wgpu::TextureFormat::Rgba16Float);

        let cubemap = hdr.to_cubemap(&device, &queue, 1024);

        assert_eq!(cubemap.size().depth_or_array_layers, 6);
    }

    #[test]
    fn should_create_brdflut_texture() {
        let (device, queue) = crate::get_device_and_queue();

        let brdflut = BRDFLUTBuilder::build(device, queue);

        assert_eq!(brdflut.format(), wgpu::TextureFormat::Rg16Float);
    }

    #[test]
    fn should_save_texture_to_file() {
        let (device, queue) = crate::get_device_and_queue();

        let brdflut = BRDFLUTBuilder::build(&device, queue);

        save_texture(&device, &queue, "testimage.png", &brdflut).unwrap();
    }

    #[test]
    fn skybox_manager_is_initialized() {
        let (device, queue) = crate::get_device_and_queue();
        let gpu_resource_manager = GPUResourceManager::new(&device);

        let manager = SkyboxManager::new(&device, &queue, &gpu_resource_manager);
        assert_eq!(manager.skyboxes.len(), SkyboxKind::iter().count());
    }

    #[test]
    fn should_create_skybox_from_6_images() {
        let (device, queue) = crate::get_device_and_queue();
        let gpu_resource_manager = GPUResourceManager::new(&device);

        let mut texture_manager = TextureManager::new(device.clone(), queue.clone());

        #[rustfmt::skip] let f0 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/skybox/right.png"));
        #[rustfmt::skip] let f1 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/skybox/left.png"));
        #[rustfmt::skip] let f2 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/skybox/top.png"));
        #[rustfmt::skip] let f3 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/skybox/bottom.png"));
        #[rustfmt::skip] let f4 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/skybox/front.png"));
        #[rustfmt::skip] let f5 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/skybox/back.png"));

        let cube = texture_manager.create_cubemap(
            f0,
            f1,
            f2,
            f3,
            f4,
            f5,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        );

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let skybox_bind_group_layout = gpu_resource_manager.get_layout(LayoutKind::Skybox);

        let _ = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &skybox_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&cube.view),
                },
            ],
            label: Some("skybox_bind_group"),
        });
    }
}
