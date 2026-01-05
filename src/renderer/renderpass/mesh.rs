use wgpu::IndexFormat;

use crate::{
    assets::{material_manager::MaterialManager, mesh_manager::MeshManager},
    renderer::{
        gpu_manager::GpuManager,
        gpu_renderer::{GpuMeshFrame, GpuView},
        pipeline_manager::{PipelineKind, PipelineManager},
        skybox_manager::SkyboxManager,
    },
};

pub struct MeshRenderPass<'a> {
    gpu: GpuView<'a>,
    encoder: &'a mut wgpu::CommandEncoder,
}

impl<'a> MeshRenderPass<'a> {
    pub fn new(gpu: GpuView<'a>, encoder: &'a mut wgpu::CommandEncoder) -> Self {
        Self { gpu, encoder }
    }
    pub fn render(self, queue: &Vec<GpuMeshFrame>) {
        let gpu_manager = self.gpu.gpu_mgr;
        let pipeline_manager = self.gpu.pip_mgr;
        let skybox_manager = self.gpu.skb_mgr;
        let material_manager = self.gpu.mat_mgr;
        let mesh_manager = self.gpu.mesh_mgr;
        let encoder = self.encoder;

        let clear_color = wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };

        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Mesh Render Pass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: &gpu_manager.hdr_frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                }),
                Some(wgpu::RenderPassColorAttachment {
                    view: &gpu_manager.entity_id_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                }),
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &gpu_manager.depth_view,
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
        renderpass.set_bind_group(0, &gpu_manager.per_frame_bind_group, &[]);
        renderpass.set_bind_group(3, skybox_manager.get_ibl_bindgroup(), &[]);

        for mesh in queue.iter() {
            let uniform_bind_group = mesh_manager.get_model_bindgroup(mesh.mesh_handle);
            renderpass.set_bind_group(2, uniform_bind_group, &[]);

            let vertex_buffer = mesh_manager.get_vertexbuffer(mesh.mesh_handle);
            let index_buffer = mesh_manager.get_indexbuffer(mesh.mesh_handle);
            let index_count = mesh_manager.get_indexcount(mesh.mesh_handle);
            let material = material_manager.get(&mesh.material_id);

            renderpass.set_bind_group(1, &material.bind_group, &[]);

            renderpass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
            renderpass.set_vertex_buffer(0, vertex_buffer.slice(..));
            renderpass.draw_indexed(0..index_count, 0, 0..1);
        }
    }
}
