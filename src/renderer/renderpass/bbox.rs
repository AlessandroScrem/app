pub use super::*;
pub use crate::math::*;
pub use crate::renderer::bbox_manager::BBoxVertexData;

struct GpuBoxFrame {
    pub boundingbox: BoundingBoxComponent,
    pub matrix: Mat4,
}

#[derive(Default)]
pub struct BBoxPass {
    enable: bool,
    axis_aligned: bool,
    bboxes: Vec<GpuBoxFrame>,
}

impl BBoxPass {
    pub fn new() -> Self {
        Self::default()
    }
}

impl BBoxPass {
    fn update_to_gpu(&mut self, ctx: &mut RenderContext) {
        let device = ctx.device;
        let bbox_mgr = &mut ctx.bbox_mgr;

        // recreate vertices every frame
        if self.enable {
            let vertices = self
                .bboxes
                .iter()
                .map(|b| {
                    if self.axis_aligned {
                        b.boundingbox.gen_aabb_vertices()
                    } else {
                        b.boundingbox.gen_obb_vertices(&b.matrix)
                    }
                })
                .collect::<Vec<BBoxVertexData>>();
            bbox_mgr.create_buffer(&device, &vertices);
        };
    }
}

impl RenderPass for BBoxPass {
    fn name(&self) -> &'static str {
        "BoundingboxPass"
    }

    fn prepare(
        &mut self,
        _asset_mgr: &AssetManager,
        world: &World,
        _resources: &Resources,
        _camera: &Camera,
        globals: &Globals,
        _selected: Option<Entity>,
        _input: &Input,
        ctx: &mut RenderContext,
    ) {
        self.bboxes.clear();
        self.enable = globals.bbox_enable;
        self.axis_aligned = globals.bbox_axis_aligned;

        // -------- BoundingBox --------
        let mut bbox_query = <(&BoundingBoxComponent, &GlobalModelComponent)>::query();

        for (boundingbox, global_model) in bbox_query.iter(world) {
            self.bboxes.push(GpuBoxFrame {
                boundingbox: boundingbox.clone(),
                matrix: global_model.mat,
            });
        }
        self.update_to_gpu(ctx);
    }

    fn execute(&mut self, encoder: &mut wgpu::CommandEncoder, ctx: &mut RenderContext) {
        let enable = self.enable;

        if !enable {
            return;
        }

        let gpu_manager = ctx.gpu_mgr;
        let pipeline_manager = ctx.pip_mgr;
        let bbox_mgr = &ctx.bbox_mgr;

        // Render pass
        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Axis Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &gpu_manager.hdr_frame.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        let pipeline = pipeline_manager.get_render_pipeline(PipelineKind::Lines);

        renderpass.set_pipeline(&pipeline);
        renderpass.set_bind_group(0, &gpu_manager.camera_bind_group, &[]);

        let vertexbuffer = &bbox_mgr.get_vertexbuffer();
        let count = bbox_mgr.get_count();
        renderpass.set_vertex_buffer(0, vertexbuffer.slice(0..));
        renderpass.draw(0..count, 0..1);
    }
}
