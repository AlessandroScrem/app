use super::*;

#[derive(Default)]
pub struct AxisPass {
    enable: bool,
}

impl AxisPass {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RenderPass for AxisPass {
    fn name(&self) -> &'static str {
        "AxisPass"
    }

    fn reads(&self) -> &[ResourceId] {
        &[]
    }
    fn writes(&self) -> &[ResourceId] {
        &[HDRA]
    }

    fn prepare(
        &mut self,
        _asset_mgr: &AssetManager,
        _world: &World,
        globals: &Globals,
        _selected: Option<Entity>,
        _input: &Input,
        _ctx: &mut RenderContext,
    ) {
        self.enable = globals.axis_enable;
    }

    fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        ctx: &mut RenderContext,
        _asset_mgr: &AssetManager,
    ) {
        let enable = self.enable;
        if !enable {
            return;
        }

        let gpu_manager = ctx.gpu_mgr;
        let pipeline_manager = ctx.pip_mgr;

        // Render pass
        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Axis Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: gpu_manager.get_framebuffer_view(FramebufferKind::Hdr),
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
        renderpass.set_bind_group(0, gpu_manager.get_bindgroup(BindgroupKind::Camera), &[]);
        renderpass.set_vertex_buffer(0, gpu_manager.get_buffer(BufferKind::Axis).slice(0..));
        renderpass.draw(0..6, 0..1);
    }
}
