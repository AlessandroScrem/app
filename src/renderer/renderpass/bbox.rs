use super::*;
use crate::entities::bounding_box_impl::{BBoxVertexData, VERTICES};

#[derive(Default)]
pub struct BBoxPass {
    enable: bool,
    vertexbuffer: Option<wgpu::Buffer>,
    count: u32,
}

impl BBoxPass {
    pub fn new() -> Self {
        Self::default()
    }
}

impl BBoxPass {
    fn create_buffer(&mut self, device: &wgpu::Device, vertices: Vec<BBoxVertexData>) {
        use wgpu::util::DeviceExt;
        let vertexbuffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("BBox Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        self.count = (vertices.len() * VERTICES) as u32;
        self.vertexbuffer = Some(vertexbuffer);
    }
}

impl RenderPass for BBoxPass {
    fn name(&self) -> &'static str {
        "BoundingboxPass"
    }

    fn reads(&self) -> &[ResourceId] {
        &[]
    }
    fn writes(&self) -> &[ResourceId] {
        &[HDRA]
    }

    fn prepare(
        &mut self,
        _asset_mgr: &AssetManager,
        world: &World,
        globals: &Globals,
        _selected: Option<Entity>,
        _input: &Input,
        ctx: &mut RenderContext,
    ) {
        if !globals.axis_enable {
            return;
        }

        // create unique vb every pass
        self.vertexbuffer = None;

        self.enable = globals.bbox_enable;
        let axis_aligned = globals.bbox_axis_aligned;

        // -------- BoundingBox --------
        let mut bbox_query = <(&BoundingBoxComponent, &GlobalModelComponent)>::query();

        let vertexdata = bbox_query
            .iter(world)
            .map(|(bbox, global_model)| {
                if axis_aligned {
                    bbox.gen_aabb_vertices()
                } else {
                    bbox.gen_obb_vertices(&global_model.mat)
                }
            })
            .collect::<Vec<_>>();

        if !vertexdata.is_empty() {
            self.create_buffer(ctx.device, vertexdata);
        }
    }

    fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        ctx: &mut RenderContext,
        _asset_mgr: &AssetManager,
    ) {
        if !self.enable {
            return;
        }

        let Some(vertexbuffer) = self.vertexbuffer.as_ref() else {
            return;
        };

        let gpu_manager = ctx.gpu_mgr;
        let pipeline_manager = ctx.pip_mgr;
        let count = self.count;

        // Render pass
        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Axis Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: gpu_manager.get_framebuffer_view(FramebufferKind::Hdr),
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
        renderpass.set_bind_group(0, gpu_manager.get_bindgroup(BindgroupKind::Camera), &[]);

        renderpass.set_vertex_buffer(0, vertexbuffer.slice(0..));
        renderpass.draw(0..count, 0..1);
    }
}
