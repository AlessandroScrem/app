use crate::{
    LightComponent,
    renderer::{
        gpu_manager::GpuManager,
        light_manager::LightManager,
        pipeline_manager::{PipelineKind, PipelineManager},
    },
};

use legion::{world::SubWorld, *};

#[system]
#[read_component(LightComponent)]
pub fn render_light(
    world: &mut SubWorld,
    #[resource] encoder: &mut wgpu::CommandEncoder,
    #[resource] gpu_manager: &GpuManager,
    #[resource] pipeline_manager: &PipelineManager,
    #[resource] light_manager: &LightManager,
) {
    // Render pass
    let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Light Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &gpu_manager.hdr_frame.view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &gpu_manager.depth_view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
    });

    let pipeline = pipeline_manager.get_render_pipeline(PipelineKind::Light);

    renderpass.set_pipeline(&pipeline);
    renderpass.set_bind_group(0, &gpu_manager.per_frame_bind_group, &[]);
    renderpass.set_bind_group(1, &light_manager.light_texture_bind_group, &[]);

    let mut query = <&LightComponent>::query();
    for _light in query.iter(world) {
        renderpass.draw(0..6, 0..1);
    }
}