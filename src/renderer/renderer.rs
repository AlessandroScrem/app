use super::{egui_tools, pipeline, uniform};
use crate::camera::Camera;

use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 =>Float32x3, 1 => Float32x3];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.5, 0.0],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        color: [0.0, 0.0, 1.0],
    },
];

struct VertexBuffer(wgpu::Buffer);

pub struct Renderer {
    window: Arc<Window>,
}

impl Renderer {
    pub async fn new(
        window: Arc<Window>,
        resources: &mut legion::Resources,
        world: &legion::World,
    ) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();

        let surface = instance.create_surface(window.clone()).unwrap();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats[0];
        let shader = device.create_shader_module(wgpu::include_wgsl!("../shader.wgsl"));
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let mut camera_uniform = uniform::CameraUniform::new();

        use legion::IntoQuery;
        let mut query = <legion::Read<Camera>>::query();
        for camera in query.iter(world) {
            camera_uniform.update_view_proj(&camera);
        }

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
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
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("Camera Bind Group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });
        let render_pipeline = pipeline::create_pipeline(
            &device,
            &render_pipeline_layout,
            &surface_config,
            shader,
            Vertex::desc(),
        );

        let egui_renderer =
            egui_tools::EguiRenderer::new(&device, surface_config.format, None, 1, &window);

        let vertex_buffer = VertexBuffer(device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            },
        ));

        resources.insert(camera_uniform);
        resources.insert(camera_buffer);
        resources.insert(camera_bind_group);
        resources.insert(render_pipeline);
        resources.insert(vertex_buffer);
        resources.insert(egui_renderer);
        resources.insert(surface_config);
        resources.insert(queue);
        resources.insert(surface);
        resources.insert(device);

        Self { window }
    }


    pub fn render/* <F: FnOnce(&egui::Context, &mut egui::Ui)> */(
        &self,
        // ui_callback: F,
        resources: &legion::Resources,
    ) -> Result<(), wgpu::SurfaceError> {
        let surface = resources.get_mut::<wgpu::Surface>().unwrap();
        let device = resources.get_mut::<wgpu::Device>().unwrap();
        let output = surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let clear_color = wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            };

            let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
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

            let render_pipeline = resources.get::<wgpu::RenderPipeline>().unwrap();
            let camera_bind_group = resources.get::<wgpu::BindGroup>().unwrap();
            let vertex_buffer = resources.get::<VertexBuffer>().unwrap();

            renderpass.set_pipeline(&render_pipeline);
            renderpass.set_bind_group(0, &camera_bind_group.clone(), &[]);
            renderpass.set_vertex_buffer(0, vertex_buffer.0.slice(..));
            renderpass.draw(0..3, 0..1);
        }
/* 
        {
            let surface_config = resources.get::<wgpu::SurfaceConfiguration>().unwrap();
            let screen_descriptor = ScreenDescriptor {
                size_in_pixels: [surface_config.width, surface_config.height],
                pixels_per_point: self.window.as_ref().scale_factor() as f32,
            };

            let mut egui_renderer = resources.get_mut::<egui_tools::EguiRenderer>().unwrap();
            let queue = resources.get_mut::<wgpu::Queue>().unwrap();

            egui_renderer.update_ui(ui_callback);
            egui_renderer.begin_frame(&self.window);
            egui_renderer.end_frame_and_draw(
                &device,
                &queue,
                &mut encoder,
                &self.window,
                &view,
                screen_descriptor,
            );
        } */

        //submit

        if let  Some(queue) = resources.get_mut::<wgpu::Queue>(){
            queue.submit([encoder.finish()]);
            self.window.pre_present_notify();
            output.present();
        }

        Ok(())
    }
}
