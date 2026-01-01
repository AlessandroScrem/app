use legion::*;

use crate::prelude::ui::ImguiState;

#[system]
pub fn render_imgui(
    #[resource] imgui: &mut ImguiState,
    #[resource] encoder: &mut wgpu::CommandEncoder,
    #[resource] frame_view: &wgpu::TextureView,
    #[resource] device: &wgpu::Device,
    #[resource] queue: &wgpu::Queue,
) {
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
