use crate::renderer::{gpu_renderer::GpuView, pipeline_manager::PipelineKind};

pub struct AxisRenderPass<'a> {
    gpu: GpuView<'a>,
    encoder: &'a mut wgpu::CommandEncoder,
}

impl<'a> AxisRenderPass<'a> {
    pub fn new(gpu: GpuView<'a>, encoder: &'a mut wgpu::CommandEncoder) -> Self {
        Self { gpu, encoder }
    }

    pub fn render(self, enable: bool) {
        if !enable {
            return;
        }

        let gpu_manager = self.gpu.gpu_mgr;
        let pipeline_manager = self.gpu.pip_mgr;
        let encoder = self.encoder;

        // Render pass
        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Axis Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &gpu_manager.hdr_frame.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        let pipeline = pipeline_manager.get_render_pipeline(PipelineKind::Lines);

        renderpass.set_pipeline(&pipeline);
        renderpass.set_bind_group(0, &gpu_manager.camera_bind_group, &[]);
        renderpass.set_vertex_buffer(0, gpu_manager.axis_vertexbuffer.slice(0..));
        renderpass.draw(0..6, 0..1);
    }
}
