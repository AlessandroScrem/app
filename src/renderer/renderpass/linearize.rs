use crate::renderer::{gpu_renderer::GpuView, pipeline_manager::PipelineKind};

pub struct LinerizeRenderPass<'a> {
    gpu: GpuView<'a>,
    encoder: &'a mut wgpu::CommandEncoder,
}

impl<'a> LinerizeRenderPass<'a> {
    pub fn new(gpu: GpuView<'a>, encoder: &'a mut wgpu::CommandEncoder) -> Self {
        Self { gpu, encoder }
    }

    pub fn render(self, frame_view: &wgpu::TextureView) {
        let gpu_manager = self.gpu.gpu_mgr;
        let pipeline_manager = self.gpu.pip_mgr;
        let encoder = self.encoder;

        // Render pass
        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Hdr Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: frame_view,
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

        let pipeline = pipeline_manager.get_render_pipeline(PipelineKind::Hdr);

        renderpass.set_pipeline(&pipeline);
        renderpass.set_bind_group(0, &gpu_manager.hdr_frame.hdr_bind_group, &[]);
        renderpass.set_bind_group(1, &gpu_manager.per_frame_bind_group, &[]);
        renderpass.draw(0..3, 0..1);
    }
}
