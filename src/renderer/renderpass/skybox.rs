use super::*;

#[derive(Default)]
pub struct SkyboxPass {
}

impl SkyboxPass {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RenderPass for SkyboxPass {
    fn name(&self) -> &'static str {
        "SkyboxPass"
    }

    fn reads(&self) -> &[ResourceId] {
        &[]
    }
    fn writes(&self) -> &[ResourceId] {
        &[ResourceId::HDRA, ResourceId::DEPTH]
    }
    
    fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        ctx: &mut RenderContext,
        frame: &FrameData,
    ) {

        if frame.skybox_enable.is_none() {
            return;
        }

        let skybox_blur = frame.skybox_enable.is_some_and(|b| b == true);

        let gpu_manager = ctx.gpu_mgr;
        let pipeline_manager = ctx.pip_mgr;
        let skybox_manager = ctx.skb_mgr;

        // Render pass
        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Skybox Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: gpu_manager.get_framebuffer_view(FramebufferKind::Hdr),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: gpu_manager.get_framebuffer_view(FramebufferKind::Depth),
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
        let skybox_bind_group = skybox_manager.get_skybox(skybox_blur);

        renderpass.set_pipeline(&pipeline);
        renderpass.set_bind_group(0, gpu_manager.get_bindgroup(BindgroupKind::Perframe), &[]);
        renderpass.set_bind_group(1, skybox_bind_group, &[]);
        renderpass.draw(0..36, 0..1);
    }
}
