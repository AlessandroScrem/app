// passi per creare un IBL

// CreateBRDFintegrationMap();
// CubemapFromHDR(skybox);
// CreateIrradiance(skybox);
// CreatePrefilterMap(skybox);

use crate::{
    renderer::pipeline_manager::PipelineManager, resources::gpu_manager::GPUResourceManager,
};

pub fn create_equirectangular_to_cubemap_pipeline(
    device: &wgpu::Device,
    gpu_resource_manager: &GPUResourceManager,
    pipeline_manager: &mut PipelineManager,
    texture_format: wgpu::TextureFormat,
) {
    let layout_map = gpu_resource_manager.bind_group_layouts.lock().unwrap();

    let layouts: Vec<&wgpu::BindGroupLayout> = vec![
        layout_map.get("camera").unwrap(),          // 0
        layout_map.get("equirectangular").unwrap(), // 1
    ];

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &layouts,
        push_constant_ranges: &[],
    });

    let shader =
        device.create_shader_module(wgpu::include_wgsl!("equirectangular_to_cubemap.wgsl"));

    let buffers = &[];

    let pipeline_desc = crate::renderer::pipeline_manager::PipelineDesc {
        depth_stencil: None,
        ..Default::default()
    };

    pipeline_manager.add_pipeline(
        "equirectangular_to_cubemap_pipeline",
        &device,
        render_pipeline_layout,
        buffers,
        shader,
        texture_format,
        pipeline_desc,
    );
}

pub fn create_equirect_bind_group(
    device: &wgpu::Device,
    gpu_resource_manager: &GPUResourceManager,
    view: &wgpu::TextureView,
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

    let equirect_bind_group_layout = gpu_resource_manager.get_layout("equirectangular");
    let equirect_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &equirect_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(view),
            },
        ],
        label: Some("equirect_bind_group"),
    });

    equirect_bind_group
}

pub fn create_dest_cube_texture(
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

pub fn create_texture_views(cube_texture: &wgpu::Texture) -> Vec<wgpu::TextureView> {
    let cube_face_views: Vec<wgpu::TextureView> = (0..6)
        .map(|i| {
            cube_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some(&format!("Cubemap Face {}", i)),
                dimension: Some(wgpu::TextureViewDimension::D2),
                base_array_layer: i,
                base_mip_level: 0,
                mip_level_count: Some(1),
                array_layer_count: Some(1),
                aspect: wgpu::TextureAspect::All,
                format: None,
                ..Default::default()
            })
        })
        .collect();
    cube_face_views
}

pub fn create_camera_views() -> Vec<cgmath::Matrix4<f32>> {
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

pub fn update_camera(
    queue: &wgpu::Queue,
    camera_uniform_buffer: &wgpu::Buffer,
    cam_view: cgmath::Matrix4<f32>,
) {
    let cam_proj = cgmath::perspective(cgmath::Deg::<f32>(90.0), 1.0, 0.1, 10.0);

    let updated_uniforms = crate::prelude::CameraUniform {
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

pub fn render_to_cubemap(
    encoder: &mut wgpu::CommandEncoder,
    pipeline_manager: &PipelineManager,
    gpu_resource_manager: &GPUResourceManager,
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

    let render_pipeline = pipeline_manager
        .get_render_pipeline("equirectangular_to_cubemap_pipeline")
        .expect("expected pipeline: 'equirectangular'");

    renderpass.set_pipeline(render_pipeline);
    renderpass.set_bind_group(0, &gpu_resource_manager.camera_bind_group, &[]);
    renderpass.set_bind_group(1, equirect_bind_group, &[]);
    renderpass.draw(0..36, 0..1);
}


pub fn create_cubemap_texture_from_hdr(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline_manager: &mut PipelineManager,
    gpu_resource_manager: &GPUResourceManager,
) -> wgpu::Texture {
    // CubemapFromHDR(skybox);
    // -- create equirect texture
    // -- create equirect pipeline
    // -- create dest cube texture
    // -- create dest cube 6 textureview
    // -- set 6 camera view matrix each for cube side
    // -- render equirect to cubemap 6faces

    let src_format = wgpu::TextureFormat::Rgba16Float;
    let dest_format = wgpu::TextureFormat::Rgba8Unorm;
    let width = 1024;
    let height = 1024;

    // create source: hdr texture HDR
    #[rustfmt::skip] let f0 = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/core/clarens_night_02_2k.hdr"));
    let buffer = std::fs::read(f0).unwrap();
    let hdr_texture = crate::assets::texture::Texture::new(device, queue, &buffer, src_format);

    // create dest: cubemap texture LDR (TODO: add tonemap)

    create_equirectangular_to_cubemap_pipeline(
        &device,
        gpu_resource_manager,
        pipeline_manager,
        dest_format,
    );

    let dest_texture = create_dest_cube_texture(&device, width, height, dest_format);
    let cube_dest_views = create_texture_views(&dest_texture);

    // create bindgroup for hdr attachement
    let equirect_bind_group =
        create_equirect_bind_group(&device, &gpu_resource_manager, &hdr_texture.view);

    // create camera matrix/views
    let camera_uniform_buffer = &gpu_resource_manager.camera_uniform_buffer;
    let camera_views = create_camera_views();

    // render faces
    for i in 0..6 {
        let mut encoder = device.create_command_encoder(&Default::default());
        update_camera(&queue, camera_uniform_buffer, camera_views[i]);

        render_to_cubemap(
            &mut encoder,
            &pipeline_manager,
            &gpu_resource_manager,
            &equirect_bind_group,
            &cube_dest_views[i],
        );
        queue.submit([encoder.finish()]);
    }

    dest_texture
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::renderer::pipeline_manager::PipelineManager;

    #[test]
    fn should_create_cubemap_from_hdr() {

        let (device, queue) = crate::get_device_and_queue();
        let mut pipeline_manager = PipelineManager::new();
        let gpu_manager = GPUResourceManager::new(&device);

        let texture = create_cubemap_texture_from_hdr(&device, &queue, &mut pipeline_manager, &gpu_manager);

        assert_eq!(texture.size().depth_or_array_layers, 6);
    }
}
