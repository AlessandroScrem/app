use crate::ecs::components::bounding_box_impl::{BBoxVertexData, VERTICES};

use super::*;

pub struct BBoxData {
    pub vertexbuffer: wgpu::Buffer,
    pub count: u32,
}

#[derive(Default)]
pub struct BoundingboxPass {}

impl BoundingboxPass {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RenderPass for BoundingboxPass {
    fn name(&self) -> &'static str {
        "BoundingboxPass"
    }

    fn reads(&self) -> &[ResourceId] {
        &[]
    }
    fn writes(&self) -> &[ResourceId] {
        &[ResourceId::HDR]
    }

    fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        ctx: &mut RenderContext,
        frame: &FrameData,
    ) {
        if  frame.bbox_vertexdata.is_empty()   {
            return;
        };

        let bufferdata = create_buffer(ctx.device, &frame.bbox_vertexdata);

        let count = bufferdata.count;
        let vertexbuffer = &bufferdata.vertexbuffer;

        let gpu_manager = ctx.gpu_mgr;
        let pipeline_manager = ctx.pip_mgr;

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
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        let pipeline = pipeline_manager.get_render_pipeline(PipelineKind::Lines);

        renderpass.set_pipeline(&pipeline);
        renderpass.set_bind_group(0, gpu_manager.get_bindgroup(BindgroupKind::Camera), &[]);

        renderpass.set_vertex_buffer(0, vertexbuffer.slice(0..));
        renderpass.draw(0..count, 0..1);
    }
}

fn create_buffer(device: &wgpu::Device, vertices: &Vec<BBoxVertexData>) -> BBoxData {
    use wgpu::util::DeviceExt;
    let vertexbuffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("BBox Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let count = (vertices.len() * VERTICES) as u32;

    BBoxData {
        count,
        vertexbuffer,
    }
}
