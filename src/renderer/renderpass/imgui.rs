use crate::{prelude::ui::ImguiLayer, renderer::gpu_renderer::GpuView};

pub struct ImguiRenderPass<'a> {
    gpu: GpuView<'a>,
    encoder: &'a mut wgpu::CommandEncoder,
}

impl<'a> ImguiRenderPass<'a> {
    pub fn new(gpu: GpuView<'a>, encoder: &'a mut wgpu::CommandEncoder) -> Self {
        Self { gpu, encoder }
    }

    pub fn render(self, frame_view: &wgpu::TextureView, imgui: &mut ImguiLayer) {

        let device = self.gpu.device;
        let queue = self.gpu.queue;
        let encoder = self.encoder;

        // Render pass
        let mut pass = {
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ImGui Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: frame_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // non cancellare la scena
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            })
        };

        let draw_data = imgui.context.render();
        imgui
            .renderer
            .render(draw_data, queue, device, &mut pass)
            .unwrap();
    }
}
