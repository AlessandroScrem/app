use crate::renderer::{LightUniform, gpu_renderer::GpuView, pipeline_manager::PipelineKind};

pub struct LightRenderPass<'a> {
    gpu: GpuView<'a>,
    encoder: &'a mut wgpu::CommandEncoder,
}

impl<'a> LightRenderPass<'a> {
    pub fn new(gpu: GpuView<'a>, encoder: &'a mut wgpu::CommandEncoder) -> Self {
        Self { gpu, encoder }
    }
    pub fn render(self, queue: &Vec<LightUniform>) {
        let gpu_manager = self.gpu.gpu_mgr;
        let pipeline_manager = self.gpu.pip_mgr;
        let light_manager = self.gpu.light_mgr;
        let encoder = self.encoder;

        // Render pass
        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Light Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &gpu_manager.hdr_frame.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &gpu_manager.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        let pipeline = pipeline_manager.get_render_pipeline(PipelineKind::Light);

        renderpass.set_pipeline(&pipeline);
        renderpass.set_bind_group(0, &gpu_manager.per_frame_bind_group, &[]);
        renderpass.set_bind_group(1, &light_manager.light_texture_bind_group, &[]);

        for _light in queue.iter() {
            renderpass.draw(0..6, 0..1);
        }
    }
}
