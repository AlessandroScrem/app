use crate::material_manager::MaterialId;
pub use super::*;

struct GpuMeshFrame {
    pub mesh_handle: usize,
    pub material_id: MaterialId,
    pub model: ModelUniform,
}

#[derive(Default)]
pub struct MeshPass {
    meshes: Vec<GpuMeshFrame>,
}

impl MeshPass {
    pub fn new() -> Self {
        Self::default()
    }
}

impl MeshPass {
    fn update_to_gpu(&mut self, ctx: &mut RenderContext) {
        let mesh_mgr = ctx.mesh_mgr;
        let mat_mgr = ctx.mat_mgr;
        let queue = ctx.queue;

        for mesh in self.meshes.iter() {
            // Material Uniform
            let material = mat_mgr.get(&mesh.material_id);
            let updated_uniforms = MaterialUniform::from(&material.material_pbr);
            queue.write_buffer(
                &material.uniform_buffer,
                0,
                bytemuck::bytes_of(&updated_uniforms),
            );

            // Model Uniform
            queue.write_buffer(
                mesh_mgr.get_model_uniform(mesh.mesh_handle),
                0,
                bytemuck::bytes_of(&mesh.model),
            );
        }
    }
}

impl RenderPass for MeshPass {
    fn name(&self) -> &'static str {
        "MeshPass"
    }

    fn prepare(
        &mut self,
        world: &World,
        _resources: &Resources,
        _camera: &Camera,
        _globals: &Globals,
        _selected: Option<Entity>,
        _input: &Input,
        ctx: &mut RenderContext,
    ) {
        self.meshes.clear();

        // -------- Mesh --------
        let mut mesh_query = <(Entity, &MeshComponent, &GlobalModelComponent)>::query();

        for (entity, mesh, global) in mesh_query.iter(world) {
            let mut model = ModelUniform::new(global.mat);
            model.entity_id = entity.as_raw_u64();
            self.meshes.push(GpuMeshFrame {
                mesh_handle: mesh.handle,
                model,
                material_id: mesh.mat_handle.clone(),
            });
        }

        self.update_to_gpu(ctx);
    }

    fn execute(&mut self, encoder: &mut wgpu::CommandEncoder, ctx: &mut RenderContext) {
        let gpu_manager = ctx.gpu_mgr;
        let pipeline_manager = ctx.pip_mgr;
        let skybox_manager = ctx.skb_mgr;
        let material_manager = ctx.mat_mgr;
        let mesh_manager = ctx.mesh_mgr;
        let meshes = &self.meshes;

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

        for mesh in meshes.iter() {
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
