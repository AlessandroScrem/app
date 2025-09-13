use crate::renderer::{
    hdr_frame::HdrFrame,
    pipeline_manager::{PipelineKind, PipelineManager},
};

use legion::*;

#[system]
pub fn hdr(
    #[resource] frame_view: &wgpu::TextureView,
    #[resource] encoder: &mut wgpu::CommandEncoder,
    #[resource] pipeline_manager: &PipelineManager,
    #[resource] hdr_frame: &HdrFrame,
) {
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
    renderpass.set_bind_group(0, &hdr_frame.hdr_bind_group, &[]);
    renderpass.draw(0..36, 0..1);
}
