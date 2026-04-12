use super::*;

use crate::renderer::drawables;

#[derive(Default)]
pub struct MeshPass {}

impl MeshPass {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RenderPass for MeshPass {
    fn name(&self) -> &'static str {
        "MeshPass"
    }

    fn reads(&self) -> &[ResourceId] {
        &[]
    }
    fn writes(&self) -> &[ResourceId] {
        &[ResourceId::HDRA, ResourceId::ENTITY, ResourceId::DEPTH]
    }

    fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        ctx: &mut RenderContext,
        frame: &FrameData,
    ) {

        let gpu_manager = ctx.gpu_mgr;
        let pipeline_manager = ctx.pip_mgr;
        let meshdraw = &frame.opaque;

        let clear_color = wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };

        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Mesh Render Pass"),
            color_attachments: &[
                // 0: opaque object
                Some(wgpu::RenderPassColorAttachment {
                    view: gpu_manager.get_framebuffer_view(FramebufferKind::Hdr),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                }),
                // 1: entity ID
                Some(wgpu::RenderPassColorAttachment {
                    view: gpu_manager.get_framebuffer_view(FramebufferKind::EntityId),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                }),
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: gpu_manager.get_framebuffer_view(FramebufferKind::Depth),
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        let render_pipeline = pipeline_manager.get_render_pipeline(PipelineKind::Pbr);

        renderpass.set_pipeline(render_pipeline);
        renderpass.set_bind_group(0, gpu_manager.get_bindgroup(BindgroupKind::Perframe), &[]);
        renderpass.set_bind_group(3, gpu_manager.get_bindgroup(BindgroupKind::Ibl), &[]);

        // Draw per material (reduce drawcall number)
        let mut drawables: Vec<_> = drawables(meshdraw, ctx.gpu_cache).collect();
        drawables.sort_by_key(|d| d.material_bg as *const _ as usize);

        let mut current_material: Option<*const _> = None;

        for mesh in drawables {
            let mat_ptr = mesh.material_bg as *const _;

            if current_material != Some(mat_ptr) {
                renderpass.set_bind_group(1, mesh.material_bg, &[]);
                current_material = Some(mat_ptr);
            }

            renderpass.set_bind_group(2, &mesh.gpu_mesh.model_bind_group, &[]);
            renderpass.set_index_buffer(mesh.gpu_mesh.indexbuffer.slice(..), IndexFormat::Uint32);
            renderpass.set_vertex_buffer(0, mesh.gpu_mesh.vertexbuffer.slice(..));

            renderpass.draw_indexed(mesh.index_range.clone(), 0, 0..1);
        }
    }
}
