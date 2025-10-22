use std::sync::Arc;

use crate::renderer::{
    gpu_manager::GPUResourceManager,
    hdr_frame::HdrFrame,
    pipeline_manager::{PipelineKind, PipelineManager},
};

use legion::*;

#[system]
pub fn render_hdr_to_ldr(
    #[resource] frame_view: &wgpu::TextureView,
    #[resource] encoder: &mut wgpu::CommandEncoder,
    #[resource] pipeline_manager: &PipelineManager,
    #[resource] hdr_frame: &HdrFrame,
    #[resource] gpu_resource_manager: &Arc<GPUResourceManager>,
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
    renderpass.set_bind_group(1, &gpu_resource_manager.globals_bind_group, &[]);
    renderpass.draw(0..3, 0..1);
}
