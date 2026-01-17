pub use super::*;

#[derive(Default)]
pub struct OutlinePass {
    enable: bool,
}
impl OutlinePass {
    pub fn new()->Self{
        Self::default()
    }
}

impl RenderPass for OutlinePass {
    fn name(&self) -> &'static str {
        "OutlinePass"
    }

    fn prepare(
        &mut self,
        _world: &World,
        _resources: &Resources,
        _camera: &Camera,
        _globals: &Globals,
        selected: Option<Entity>,
        _input: &Input,
        _ctx: &mut RenderContext,
    ) {
        self.enable = selected.is_some();
    }

    fn execute(&mut self, encoder: &mut wgpu::CommandEncoder, ctx: &mut RenderContext) {
        // let pick_object = self.gpu.pickobject;
        let enable = self.enable;
        let frame_view = &ctx.target;

        if !enable {
            return;
        }

        let gpu_manager = ctx.gpu_mgr;
        let pipeline_manager = ctx.pip_mgr;

        // Render pass
        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Outline Render Pass"),
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

        let pipeline = pipeline_manager.get_render_pipeline(PipelineKind::Outline);

        renderpass.set_pipeline(&pipeline);
        renderpass.set_bind_group(0, &gpu_manager.entity_id_texture.id_bind_group, &[]);
        renderpass.set_bind_group(1, &gpu_manager.per_frame_bind_group, &[]);
        renderpass.draw(0..3, 0..1);
    }
}
