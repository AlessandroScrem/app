use crate::renderer::{gpu_renderer::GpuView, pipeline_manager::PipelineKind};

pub struct SkyboxRenderPass<'a> {
    gpu: GpuView<'a>,
    encoder: &'a mut wgpu::CommandEncoder,
}

impl<'a> SkyboxRenderPass<'a> {
    pub fn new(gpu: GpuView<'a>, encoder: &'a mut wgpu::CommandEncoder) -> Self {
        Self { gpu, encoder }
    }
    pub fn render(self, enable: bool) {
        if !enable {
            return;
        }
        
        let gpu_manager = self.gpu.gpu_mgr;
        let pipeline_manager = self.gpu.pip_mgr;
        let skybox_manager = self.gpu.skb_mgr;
        let encoder = self.encoder;

    // Render pass
    let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Skybox Render Pass"),
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

    let pipeline = pipeline_manager.get_render_pipeline(PipelineKind::Skybox);
    let skybox_bind_group = skybox_manager.get_skybox();

    renderpass.set_pipeline(&pipeline);
    renderpass.set_bind_group(0, &gpu_manager.per_frame_bind_group, &[]);
    renderpass.set_bind_group(1, skybox_bind_group, &[]);
    renderpass.draw(0..36, 0..1);
    }
}
