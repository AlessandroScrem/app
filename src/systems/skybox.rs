use std::sync::Arc;

use crate::{renderer::{
    gpu_manager::GPUResourceManager,
    gpu_renderer::DepthTexture,
    hdr_frame::HdrFrame,
    pipeline_manager::{PipelineKind, PipelineManager},
    skybox_manager::{SkyboxKind, SkyboxManager},
}, Globals};

use legion::*;

#[system]
pub fn skybox(
    #[resource] encoder: &mut wgpu::CommandEncoder,
    #[resource] gpu_resource_manager: &Arc<GPUResourceManager>,
    #[resource] pipeline_manager: &PipelineManager,
    #[resource] depth_texture: &DepthTexture,
    #[resource] skybox_manager: &SkyboxManager,
    #[resource] hdr_texture: &HdrFrame,
    #[resource] globals: &Globals
) {
    if !globals.skybox_enable {
        return;
    }

    // Render pass
    let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Skybox Render Pass"),
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

    let pipeline = pipeline_manager.get_render_pipeline(PipelineKind::Skybox);
    let skybox_bind_group = skybox_manager.get_skybox(SkyboxKind::Default);

    renderpass.set_pipeline(&pipeline);
    renderpass.set_bind_group(0, &gpu_resource_manager.camera_bind_group, &[]);
    renderpass.set_bind_group(1, skybox_bind_group, &[]);
    renderpass.draw(0..36, 0..1);
}
