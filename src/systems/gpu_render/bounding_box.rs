use std::sync::Arc;

use crate::{
    BoundingBoxComponent, Globals,
    renderer::{
        bbox_manager::{BBoxManager}, gpu_manager::GPUResourceManager, hdr_frame::HdrFrame, pipeline_manager::{PipelineKind, PipelineManager}
    },
};

use legion::{world::SubWorld, *};

#[system]
#[read_component(BoundingBoxComponent)]
#[read_component(Entity)]
pub fn render_bounding_box(
    world: &mut SubWorld,
    #[resource] encoder: &mut wgpu::CommandEncoder,
    #[resource] device: &wgpu::Device,
    #[resource] gpu_resource_manager: &Arc<GPUResourceManager>,
    #[resource] pipeline_manager: &PipelineManager,
    #[resource] hdr_texture: &HdrFrame,
    #[resource] globals: &Globals,
    #[resource] bbox_manager: &mut BBoxManager,
) {
    if !globals.bbox_enable {
        return;
    }

    // Render pass
    let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Axis Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &hdr_texture.view,
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
    renderpass.set_bind_group(0, &gpu_resource_manager.camera_bind_group, &[]);

    let mut bbox_query = <(Entity, &BoundingBoxComponent)>::query();

    
    for (entity, _bbox) in bbox_query.iter(world) {
        let vertexbuffer = bbox_manager.get_or_create(&device, *entity);
        renderpass.set_vertex_buffer(0, vertexbuffer.slice(0..));
        renderpass.draw(0..24, 0..1);
    }
}

