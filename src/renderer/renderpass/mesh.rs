pub use super::*;
use crate::{assets::MaterialId, assets::MeshId};

struct GpuMeshFrame {
    // pub entity: EntityId,
    pub mesh_handle: MeshId,
    pub submesh: u32,
    pub index_range: std::ops::Range<u32>,
    pub material: MaterialId,
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
    fn update_to_gpu(&mut self, ctx: &mut RenderContext, asst_mgr: &AssetManager) {
        let mesh_cache = &ctx.gpu_cache.mesh;
        let material_cache = &ctx.gpu_cache.material;
        let queue = ctx.queue;

        // Material Uniform
        for mesh in self.meshes.iter() {
            if let Some(material_desc) = asst_mgr.materials.get(mesh.material) {
                let updated_uniform = MaterialUniform::from(material_desc);
                material_cache.update(&mesh.material, queue, &updated_uniform);
            }

            // Model Uniform
            mesh_cache.update(&mesh.mesh_handle, queue, &mesh.model);
        }
    }
}

impl RenderPass for MeshPass {
    fn name(&self) -> &'static str {
        "MeshPass"
    }

    fn prepare(
        &mut self,
        asset_mgr: &AssetManager,
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
            let submeshes = &asset_mgr.meshes.get(mesh.handle).unwrap().submeshes;
            for (i, submesh) in submeshes.iter().enumerate() {
                self.meshes.push(GpuMeshFrame {
                    mesh_handle: mesh.handle,
                    model,
                    material: submesh.material,
                    submesh: i as u32,
                    index_range: submesh.index_range.clone(),
                });
            }
        }

        self.update_to_gpu(ctx, asset_mgr);
    }

    fn execute(&mut self, encoder: &mut wgpu::CommandEncoder, ctx: &mut RenderContext) {
        let gpu_cache = &ctx.gpu_cache;
        let gpu_manager = ctx.gpu_mgr;
        let pipeline_manager = ctx.pip_mgr;
        let skybox_manager = ctx.skb_mgr;
        // let mesh_manager = ctx.mesh_mgr;
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
            if let Some(gpu_mesh) = gpu_cache.mesh.get(&mesh.mesh_handle) {
                if let Some(gpu_material) = gpu_cache.material.get(&mesh.material) {
                    let uniform_bind_group = &gpu_mesh.model_bind_group;
                    let vertex_buffer = &gpu_mesh.vertexbuffer;
                    let index_buffer = &gpu_mesh.indexbuffer;
                    let index_range = mesh.index_range.clone();
                    if let Some(material_bind_group) = &gpu_material.bind_group {
                        renderpass.set_bind_group(2, uniform_bind_group, &[]);
                        renderpass.set_bind_group(1, material_bind_group, &[]);

                        renderpass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
                        renderpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                        renderpass.draw_indexed(index_range, 0, 0..1);
                    }
                }
            }
        }
    }
}
