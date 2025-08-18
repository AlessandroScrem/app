use std::sync::Arc;

use crate::{
    LightComponent,
    renderer::{
        gpu_renderer::DepthTexture, light_manager::LightManager, pipeline_manager::PipelineManager,
    },
    resources::gpu_manager::GPUResourceManager,
};

use legion::{world::SubWorld, *};

#[system]
#[read_component(LightComponent)]
pub fn light(
    world: &mut SubWorld,
    #[resource] frame_view: &wgpu::TextureView,
    #[resource] encoder: &mut wgpu::CommandEncoder,
    #[resource] gpu_resource_manager: &Arc<GPUResourceManager>,
    #[resource] pipeline_manager: &PipelineManager,
    #[resource] depth_texture: &DepthTexture,
    #[resource] light_manager: &LightManager,
) {
    // Render pass
    let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Light Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: frame_view,
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

    let pipeline = pipeline_manager.get_render_pipeline("light").unwrap();

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
pub fn update_trnsform(
    light: &LightComponent,
    #[resource] queue: &wgpu::Queue,
    #[resource] light_manager: &LightManager,
) {
    queue.write_buffer(
        &light_manager.light_uniform_buffer,
        0,
        bytemuck::bytes_of(&light.data),
    );
}
