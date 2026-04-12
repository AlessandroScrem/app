use super::*;

#[derive(Default)]
pub struct LightPass {
}

impl LightPass {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RenderPass for LightPass {
    fn name(&self) -> &'static str {
        "LightPass"
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
        let lights = &frame.lights;
        let gpu_manager = ctx.gpu_mgr;
        let pipeline_manager = ctx.pip_mgr;

        // Render pass
        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Light Render Pass"),
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

        let pipeline = pipeline_manager.get_render_pipeline(PipelineKind::Light);
        let light_texture_bind_group = gpu_manager.get_bindgroup(BindgroupKind::LightTexture);

        renderpass.set_pipeline(&pipeline);
        renderpass.set_bind_group(0, gpu_manager.get_bindgroup(BindgroupKind::Perframe), &[]);
        renderpass.set_bind_group(1, light_texture_bind_group, &[]);

        for _light in lights.iter() {
            renderpass.draw(0..6, 0..1);
        }
    }
}
