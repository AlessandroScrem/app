use std::sync::Arc;

use crate::{
    LightComponent,
    renderer::{
        gpu_manager::GPUResourceManager,
        gpu_renderer::DepthTexture,
        hdr_frame::HdrFrame,
        light_manager::LightManager,
        pipeline_manager::{PipelineKind, PipelineManager},
    },
};

use legion::{world::SubWorld, *};

#[system]
#[read_component(LightComponent)]
pub fn light(
    world: &mut SubWorld,
    #[resource] encoder: &mut wgpu::CommandEncoder,
    #[resource] gpu_resource_manager: &Arc<GPUResourceManager>,
    #[resource] pipeline_manager: &PipelineManager,
    #[resource] depth_texture: &DepthTexture,
    #[resource] light_manager: &LightManager,
    #[resource] hdr_texture: &HdrFrame,
) {
    // Render pass
    let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Light Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &hdr_texture.view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &depth_texture.0,
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
    renderpass.set_bind_group(0, &gpu_resource_manager.camera_bind_group, &[]);
    renderpass.set_bind_group(1, &light_manager.light_uniform_bind_group, &[]);
    renderpass.set_bind_group(2, &light_manager.light_texture_bind_group, &[]);

    let mut query = <&LightComponent>::query();
    for _light in query.iter(world) {
        renderpass.draw(0..6, 0..1);
    }
}

#[system(for_each)]
#[filter(maybe_changed::<LightComponent>())]
pub fn update_transform(
    light: &LightComponent,
    #[resource] queue: &wgpu::Queue,
    #[resource] light_manager: &LightManager,
) {
    // println!("Light maybe_changed");
    queue.write_buffer(
        &light_manager.light_uniform_buffer,
        0,
        bytemuck::bytes_of(&light.data),
    );
}
