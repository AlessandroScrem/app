use crate::{
    gpu::{GpuCache, GpuMesh},
    renderer::MeshDraw,
};

use super::*;

pub struct MeshDrawable<'a> {
    pub gpu_mesh: &'a GpuMesh,
    pub material_bg: &'a wgpu::BindGroup,
    pub index_range: &'a std::ops::Range<u32>,
}

pub fn drawables<'a>(
    mesh_draw: &'a [MeshDraw],
    gpu_cache: &'a GpuCache,
) -> impl Iterator<Item = MeshDrawable<'a>> + 'a {
    mesh_draw.iter().filter_map(move |md| {
        let gpu_mesh = gpu_cache.mesh.get(&md.mesh)?;
        let material = gpu_cache.material.get(&md.material)?;
        let material_bg = material.bind_group.as_ref()?;

        Some(MeshDrawable {
            gpu_mesh,
            material_bg,
            index_range: &md.submesh_index_range,
        })
    })
}

pub enum MeshPassMode {
    Opaque,
    Transmission,
}

pub struct MeshPassConfig {
    pub hdr_load: wgpu::LoadOp<wgpu::Color>,
    pub entity_load: wgpu::LoadOp<wgpu::Color>,
    pub depth_load: wgpu::LoadOp<f32>,
    pub mode: MeshPassMode,
}

pub struct MeshPass {
    config: MeshPassConfig,
}

impl MeshPass {
    pub fn opaque() -> Self {
        let clear_color = wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };
        Self {
            config: MeshPassConfig {
                hdr_load: wgpu::LoadOp::Clear(clear_color),
                entity_load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                depth_load: wgpu::LoadOp::Clear(1.0),
                mode: MeshPassMode::Opaque,
            },
        }
    }
    pub fn transmission() -> Self {
        Self {
            config: MeshPassConfig {
                hdr_load: wgpu::LoadOp::Load,
                entity_load: wgpu::LoadOp::Load,
                depth_load: wgpu::LoadOp::Load,
                mode: MeshPassMode::Transmission,
            },
        }
    }
}

impl RenderPass for MeshPass {
    fn name(&self) -> &'static str {
        match self.config.mode {
            MeshPassMode::Opaque => "MeshPass Opaque",
            MeshPassMode::Transmission => "MeshPass Transmission",
        }
    }

    fn reads(&self) -> &[ResourceId] {
        match self.config.mode {
            MeshPassMode::Opaque => &[],
            MeshPassMode::Transmission => &[ResourceId::OPAQUE]
        }
    }
    fn writes(&self) -> &[ResourceId] {
        &[ResourceId::HDR, ResourceId::OPAQUE, ResourceId::ENTITY, ResourceId::DEPTH]
    }

    fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        ctx: &mut RenderContext,
        frame: &FrameData,
    ) {
        let meshdraw = match self.config.mode {
            MeshPassMode::Opaque => &frame.opaque,
            MeshPassMode::Transmission => &frame.transmission,
        };

        let gpu_manager = ctx.gpu_mgr;
        let pipeline_manager = ctx.pip_mgr;

        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Mesh Render Pass"),
            color_attachments: &[
                // 0: opaque object
                Some(wgpu::RenderPassColorAttachment {
                    view: gpu_manager.get_framebuffer_view(FramebufferKind::Hdr),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: self.config.hdr_load,
                        store: wgpu::StoreOp::Store,
                    },
                }),
                // 1: entity ID
                Some(wgpu::RenderPassColorAttachment {
                    view: gpu_manager.get_framebuffer_view(FramebufferKind::EntityId),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: self.config.entity_load,
                        store: wgpu::StoreOp::Store,
                    },
                }),
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: gpu_manager.get_framebuffer_view(FramebufferKind::Depth),
                depth_ops: Some(wgpu::Operations {
                    load: self.config.depth_load,
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
        renderpass.set_bind_group(3, gpu_manager.get_bindgroup(BindgroupKind::PbrMap), &[]);

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
