mod egui_tools;
mod pipeline;
mod uniform;

use std::fmt;

use cgmath::Point3;
use egui_tools::EguiRenderer;
use egui_wgpu::ScreenDescriptor;
use egui_winit::EventResponse;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

use crate::prelude::*;

pub struct Renderer {
    size: PhysicalSize<u32>,
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    camera_bind_group: wgpu::BindGroup,
    camera_uniform: uniform::CameraUniform,
    camera_buffer: wgpu::Buffer,
    egui_renderer: EguiRenderer,
}

struct DisplayPoint3(Point3<f32>);

impl fmt::Display for DisplayPoint3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:.2}, {:.2}, {:.2})", self.0.x, self.0.y, self.0.z)
    }
}

impl Renderer {
    pub async fn new(
        window: Arc<Window>,
        camera: &Camera,
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
        camera_uniform.update_view_proj(&camera);

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
        let render_pipeline =
            pipeline::create_pipeline(&device, &render_pipeline_layout, &surface_config, shader);

        let egui_renderer = EguiRenderer::new(&device, surface_config.format, None, 1, &window);

        Self {
            size,
            window,
            device,
            queue,
            surface,
            surface_config,
            camera_bind_group,
            render_pipeline,
            camera_uniform,
            camera_buffer,
            egui_renderer,
        }
    }

    pub fn get_window(&self) -> Arc<Window> {
        self.window.clone()
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    pub fn update(&self, _dt: instant::Duration) {
        // Update logic here
    }

    pub fn handle_input(&mut self, event: &WindowEvent)-> EventResponse {
        self.egui_renderer.handle_input(&self.window, event)
    }

    pub fn update_camera_buffer(&mut self, camera: &Camera) {
        self.camera_uniform
            .update_view_proj(camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    pub fn render(&mut self, camera: &mut Camera) -> Result<(), wgpu::SurfaceError> {
        let outpot = self.surface.get_current_texture()?;

        let view = outpot
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
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

            renderpass.set_pipeline(&self.render_pipeline);
            renderpass.set_bind_group(0, &self.camera_bind_group, &[]);
            renderpass.draw(0..3, 0..1);
        }

        {
            self.egui_renderer.begin_frame(&self.window);
            let screen_descriptor = ScreenDescriptor {
                size_in_pixels: [self.surface_config.width, self.surface_config.height],
                pixels_per_point: self.window.as_ref().scale_factor() as f32 ,
            };

            egui::Window::new("winit + egui + wgpu says hello!")
                .resizable(true)
                .vscroll(true)
                .default_open(false)
                .show(self.egui_renderer.context(), |ui| {
                    ui.label("Label!");
                    ui.label(format!("Camera position: ({})", DisplayPoint3(camera.get_position())));
                    let mut fov: f32 = camera.get_fov().0;
                    if ui.add(egui::Slider::new(&mut fov, 0.1..=179.0).text("Fov")).changed() {
                        camera.set_fov(cgmath::Deg(fov));
                        println!("Fov: {:?}", camera.get_fov());
                    };
                });

            self.egui_renderer.end_frame_and_draw(
                &self.device,
                &self.queue,
                &mut encoder,
                &self.window,
                &view,
                screen_descriptor,
            );
        }

        //submit
        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        outpot.present();

        Ok(())
    }
}
