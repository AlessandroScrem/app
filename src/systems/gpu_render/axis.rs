use crate::{
    Globals,
    renderer::{
        gpu_manager::GpuManager,
        pipeline_manager::{PipelineKind, PipelineManager},
    },
};

use legion::*;

#[system]
pub fn render_axis(
    #[resource] encoder: &mut wgpu::CommandEncoder,
    #[resource] gpu_manager: &GpuManager,
    #[resource] pipeline_manager: &PipelineManager,
    #[resource] globals: &Globals,
) {
    if !globals.axis_enable {
        return;
    }

    // Render pass
    let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Axis Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &gpu_manager.hdr_frame.view,
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
    renderpass.set_bind_group(0, &gpu_manager.camera_bind_group, &[]);
    renderpass.set_vertex_buffer(0, gpu_manager.axis_vertexbuffer.slice(0..));
    renderpass.draw(0..6, 0..1);
}
