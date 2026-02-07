use super::*;

#[derive(Default)]
pub struct ImguiPass {}

impl ImguiPass {
    pub fn new() -> Self {
        Self {}
    }
}

impl RenderPass for ImguiPass {
    fn name(&self) -> &'static str {
        "ImguiPass"
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

    fn execute(&mut self, _encoder: &mut wgpu::CommandEncoder, _ctx: &mut RenderContext, frame: &FrameDrawable) {
        // let imgui = &mut ctx.imgui;
        // let device = ctx.device;
        // let queue = ctx.queue;
        // let frame_view = &ctx.target;

        // // Render pass
        // let mut pass = {
        //     encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        //         label: Some("ImGui Pass"),
        //         color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        //             view: frame_view,
        //             resolve_target: None,
        //             ops: wgpu::Operations {
        //                 load: wgpu::LoadOp::Load,
        //                 store: wgpu::StoreOp::Store,
        //             },
        //         })],
        //         depth_stencil_attachment: None,
        //         ..Default::default()
        //     })
        // };

        // let draw_data = imgui.context.render();
        // imgui
        //     .renderer
        //     .render(draw_data, queue, device, &mut pass)
        //     .unwrap();
    }
}
