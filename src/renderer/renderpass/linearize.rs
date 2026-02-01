pub use super::*;

#[derive(Default)]
pub struct LinearizePass {}

impl LinearizePass {
    pub fn new()->Self{
        Self::default()
    }
}

impl RenderPass for LinearizePass {
    fn name(&self) -> &'static str {
        "LinearizePass"
    }
    fn prepare(
        &mut self,
        _asset_mgr: &AssetManager,
        _world: &World,
        _resources: &Resources,
        _camera: &Camera,
        _globals: &Globals,
        _selected: Option<Entity>,
        _input: &Input,
        _ctx: &mut RenderContext,
    ) {
    }

    fn execute(&mut self, encoder: &mut wgpu::CommandEncoder, ctx: &mut RenderContext) {
        let gpu_manager = ctx.gpu_mgr;
        let pipeline_manager = ctx.pip_mgr;
        let frame_view = &ctx.target;

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

