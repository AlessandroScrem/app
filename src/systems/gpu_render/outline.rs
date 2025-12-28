use crate::{
    picking::PickObject,
    renderer::{
        gpu_manager::GpuManager,
        pipeline_manager::{PipelineKind, PipelineManager},
    },
};

use legion::*;

#[system]
pub fn render_outline(
    #[resource] encoder: &mut wgpu::CommandEncoder,
    #[resource] gpu_manager: &GpuManager,
    #[resource] pipeline_manager: &PipelineManager,
    #[resource] frame_view: &wgpu::TextureView,
    #[resource] pick_object: &PickObject,
) {
    if pick_object.selected.is_none() {
        return;
    }

    // Render pass
    let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Outline Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &frame_view,
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
