use std::sync::Arc;
use winit::window::Window;

use crate::renderer::pipeline_manager;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 =>Float32x3, 1 => Float32x3];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub const VERTICES: &[Vertex] = &[
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

pub struct VertexBuffer(pub wgpu::Buffer);

pub struct Renderer {
    window: Arc<Window>,
}

impl Renderer {
    pub async fn new(
        window: Arc<Window>,
        resources: &mut legion::Resources,
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

        let gpu_manager = crate::resources::gpu_manager::GPUResourceManager::new(&device);
        let pipeline_manager = crate::renderer::pipeline_manager::PipelineManager::new();

        resources.insert(surface_config);
        resources.insert(pipeline_manager);
        resources.insert(gpu_manager);
        resources.insert(queue);
        resources.insert(surface);
        resources.insert(device);

        Self { window }
    }

    pub fn render(
        &self,
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

            let gpu_manager = resources.get::<crate::resources::gpu_manager::GPUResourceManager>().unwrap();
            let pipeline_manager = resources.get::<crate::renderer::pipeline_manager::PipelineManager>().unwrap();

            let render_pipeline = pipeline_manager.get_render_pipeline("default").unwrap();

            let camera_bind_group = &gpu_manager.camera_bind_group;
            let vertex_buffer = &gpu_manager.vertex_buffer;


            renderpass.set_pipeline(render_pipeline);
            renderpass.set_bind_group(0, &camera_bind_group.clone(), &[]);
            renderpass.set_vertex_buffer(0, vertex_buffer.0.slice(..));
            renderpass.draw(0..3, 0..1);
        }

        //submit
        if let Some(queue) = resources.get_mut::<wgpu::Queue>() {
            queue.submit([encoder.finish()]);
            self.window.pre_present_notify();
            output.present();
        }

        Ok(())
    }
}
