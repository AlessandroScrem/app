use crate::{ecs::entity_id::EntityId, gpu::shadow_manager::GpuShadow};
use crate::math::*;

use super::*;

pub struct ShadowPass {}

impl RenderPass for ShadowPass {
    fn name(&self) -> &'static str {
        "ShadowPass Opaque"
    }

    fn reads(&self) -> &[ResourceId] {
        &[]
    }

    fn writes(&self) -> &[ResourceId] {
        &[ResourceId::SHADOWMAP]
    }

    fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        ctx: &mut RenderContext,
        frame: &FrameData,
    ) {
        let Some(lights) = frame.lights else {
            return;
        };

        for (id, light) in lights.lights.iter().enumerate() {
            if light.cast_shadow.is_zero() {
                continue;
            }
            if let Some(shadow_map) = ctx.shadow_mgr.get_shadowmap(id) {
                build_shadowmap(encoder, ctx, frame, shadow_map);
                if let Some(entity) = frame.entity_selected {
                    if EntityId(light.entity_id) == EntityId::from(entity) {
                        convert_texture(encoder, ctx, shadow_map);
                    }
                }
            }
        }
    }
}

fn build_shadowmap(
    encoder: &mut wgpu::CommandEncoder,
    ctx: &mut RenderContext,
    frame: &FrameData,
    shadow_map: &GpuShadow,
) {
    let batches = &frame.opaque_batches;

    let gpu_manager = ctx.gpu_mgr;
    let view = shadow_map.get_view();
    let bindgroup = ctx.shadow_mgr.get_create_bg();
    let pipeline = ctx.shadow_mgr.get_pipeline();

    let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Mesh Render Pass"),
        color_attachments: &[],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        ..Default::default()
    });

    renderpass.set_pipeline(pipeline);
    renderpass.set_bind_group(0, bindgroup, &[]);

    // -------------------------------------------------
    // INSTANCE BUFFER
    // -------------------------------------------------
    renderpass.set_vertex_buffer(1, gpu_manager.get_buffer(BufferKind::Instances).slice(..));

    // -------------------------------------------------
    // DRAW
    // -------------------------------------------------
    for batch in batches {
        // ---------------------------------------------
        // GPU RESOURCES
        // ---------------------------------------------
        let Some(gpu_mesh) = ctx.gpu_cache.mesh.get(&batch.mesh) else {
            continue;
        };

        // ---------------------------------------------
        // GEOMETRY
        // ---------------------------------------------
        renderpass.set_vertex_buffer(0, gpu_mesh.vertexbuffer.slice(..));

        renderpass.set_index_buffer(gpu_mesh.indexbuffer.slice(..), IndexFormat::Uint32);

        // ---------------------------------------------
        // TRUE INSTANCING
        // ---------------------------------------------
        renderpass.draw_indexed(
            batch.submesh_index_range.clone(),
            0,
            batch.instance_start..batch.instance_start + batch.instance_count,
        );
    }
}

fn convert_texture(
    encoder: &mut wgpu::CommandEncoder,
    ctx: &mut RenderContext,
    shadow_map: &GpuShadow,
) {
    // let gpu_manager = ctx.gpu_mgr;
    let pipeline_manager = ctx.pip_mgr;
    let bindgroup = shadow_map.get_bg();
    let view = ctx.shadow_mgr.get_rgba().get_view();

    let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Convert Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            // view: gpu_manager.get_framebuffer_view(FramebufferKind::ShadowMapRgba),
            view,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: wgpu::StoreOp::Store,
            },
            resolve_target: None,
            depth_slice: None,
        })],
        ..Default::default()
    });

    let pipeline = pipeline_manager.get_render_pipeline(PipelineKind::Convert);

    renderpass.set_pipeline(pipeline);
    renderpass.set_bind_group(0, bindgroup, &[]);

    renderpass.draw(0..3, 0..1);
}
