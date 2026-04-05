use cgmath::SquareMatrix;

use crate::gpu::{GpuCache, GpuMesh};

use super::*;

struct MeshDrawable<'a> {
    gpu_mesh: &'a GpuMesh,
    material_bg: &'a wgpu::BindGroup,
    index_range: &'a std::ops::Range<u32>,
}

fn drawables<'a>(
    assets: &'a AssetManager,
    gpu_cache: &'a GpuCache,
) -> impl Iterator<Item = MeshDrawable<'a>> + 'a {
    gpu_cache.mesh.keys().flat_map(move |mesh_id| {
        // Option → Iterator
        gpu_cache
            .mesh
            .get(&mesh_id)
            .into_iter() // trasforma Some -> iteratore, None -> empty iter
            .flat_map(move |gpu_mesh| {
                assets
                    .meshes
                    .get(mesh_id)
                    .into_iter() // stessa logica
                    .flat_map(move |mesh_desc| {
                        mesh_desc.submeshes.iter().filter_map(move |sub| {
                            
                            // Get material asset
                            let material = assets.materials.get_desc(sub.material)?;
                            // Filter material opaque
                            if material.is_transmissive() {
                                return None;
                            }

                            let gpu_material = gpu_cache.material.get(&sub.material)?;
                            let bg = gpu_material.bind_group.as_ref()?;
                            Some(MeshDrawable {
                                gpu_mesh,
                                material_bg: bg,
                                index_range: &sub.index_range,
                            })
                        })
                    })
            })
    })
}

#[derive(Default)]
pub struct MeshPass {}

impl MeshPass {
    pub fn new() -> Self {
        Self::default()
    }

    fn update_to_gpu(&mut self, asset_mgr: &AssetManager, world: &World, ctx: &mut RenderContext) {
        // -------- Mesh --------
        let mesh_cache = &ctx.gpu_cache.mesh;
        let material_cache = &ctx.gpu_cache.material;
        let queue = ctx.queue;

        let mut mesh_query = <(Entity, &MeshComponent, &GlobalModelComponent)>::query();

        for (entity, mesh, global) in mesh_query.iter(world) {
            // Model Uniform
            assert!(global.mat.determinant() > 0.0 ,"matrix determinant is negative"); 

            let mut model = ModelUniform::new(global.mat);
            model.entity_id = entity.as_raw_u64();
            mesh_cache.update(&mesh.handle, queue, &model);

            if let Some(mesh_desc) = &asset_mgr.meshes.get(mesh.handle) {
                for submesh in mesh_desc.submeshes.iter() {
                    // Material Uniform
                    if let Some(material_desc) = asset_mgr.materials.get_desc(submesh.material) {
                        let updated_uniform = MaterialUniform::from(material_desc);
                        material_cache.update(&submesh.material, queue, &updated_uniform);
                    }
                }
            }
        }
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

    fn prepare(
        &mut self,
        asset_mgr: &AssetManager,
        world: &World,
        _globals: &Globals,
        _selected: Option<Entity>,
        _input: &Input,
        ctx: &mut RenderContext,
    ) {
        self.update_to_gpu(asset_mgr, world, ctx);
    }

    fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        ctx: &mut RenderContext,
        asset_mgr: &AssetManager,
    ) {
        let gpu_manager = ctx.gpu_mgr;
        let pipeline_manager = ctx.pip_mgr;
        // let skybox_manager = ctx.skb_mgr;

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
                // 2: copy opaque for transmission map
                Some(wgpu::RenderPassColorAttachment {
                    view: gpu_manager.get_framebuffer_view(FramebufferKind::HdrOpaque),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
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
        // renderpass.set_bind_group(3, skybox_manager.get_ibl_bindgroup(), &[]);


        // Draw per submesh (Default)
        // for mesh in drawables(asset_mgr, ctx.gpu_cache) {
        //     let MeshDrawable {
        //         gpu_mesh,
        //         material_bg,
        //         index_range,
        //     } = mesh;

        //     renderpass.set_bind_group(2, &gpu_mesh.model_bind_group, &[]);
        //     renderpass.set_bind_group(1, material_bg, &[]);
        //     renderpass.set_index_buffer(gpu_mesh.indexbuffer.slice(..), IndexFormat::Uint32);
        //     renderpass.set_vertex_buffer(0, gpu_mesh.vertexbuffer.slice(..));
        //     renderpass.draw_indexed((*index_range).clone(), 0, 0..1);
        // }

        // Draw per material (reduce drawcall number)
        let mut drawables: Vec<_> = drawables(asset_mgr, ctx.gpu_cache).collect();
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

// impl RenderPassNode for MeshPass {
//     fn name(&self) -> &str {
//         "MeshPass"
//     }
//     fn reads(&self) -> &[ResourceId] {
//         &[]
//     }
//     fn writes(&self) -> &[ResourceId] {
//         &[HDRA, ENTITY, DEPTH]
//     }
// }
