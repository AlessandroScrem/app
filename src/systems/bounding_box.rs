use std::sync::Arc;

use crate::{
    BoundingBoxComponent, GlobalModelComponent, Globals,
    assets::vertexdata::LinesVertexData,
    colors,
    entities::bounding_box::BoundingBox,
    math::*,
    renderer::{
        gpu_manager::GPUResourceManager,
        hdr_frame::HdrFrame,
        pipeline_manager::{PipelineKind, PipelineManager},
    },
};

use legion::{world::SubWorld, *};

#[system]
#[read_component(BoundingBoxComponent)]
pub fn render_bounding_box(
    world: &mut SubWorld,
    #[resource] encoder: &mut wgpu::CommandEncoder,
    #[resource] gpu_resource_manager: &Arc<GPUResourceManager>,
    #[resource] pipeline_manager: &PipelineManager,
    #[resource] hdr_texture: &HdrFrame,
    #[resource] globals: &Globals,
) {
    if !globals.bbox_enable {
        return;
    }

    // Render pass
    let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Axis Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &hdr_texture.view,
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
    renderpass.set_bind_group(0, &gpu_resource_manager.camera_bind_group, &[]);

    let mut bbox_query = <&BoundingBoxComponent>::query();
    for bbox in bbox_query.iter(world) {
        renderpass.set_vertex_buffer(0, bbox.vertex_buffer.slice(0..));
        renderpass.draw(0..24, 0..1);
    }
}

#[system(for_each)]
#[filter(maybe_changed::<GlobalModelComponent>())]
pub fn update_bounding_box_to_gpu(
    global_model: &GlobalModelComponent,
    bbox_component: &mut BoundingBoxComponent,
    #[resource] queue: &wgpu::Queue,
    #[resource] globals: &Globals,
) {
    if !globals.bbox_enable {
        return;
    }


    // transform local bbox and save to global_bounding_box
    bbox_component.global_bounding_box = bbox_component.bounding_box.transform(&global_model.mat);    
    let vertices = BoundingBox::gen_vertices(&bbox_component.global_bounding_box);

    queue.write_buffer(
        &bbox_component.vertex_buffer,
        0,
        bytemuck::cast_slice(&vertices.as_slice()),
    );
}

impl BoundingBox {
    pub fn gen_vertices(bbox: &BoundingBox) -> [LinesVertexData; 24] {
        /*
        bbox vertices order:
               7----------6
              /|         /|
             / |        / |
            3----------2  |
            |  |       |  |
            |  4-------|--5
            | /        | /
            |/         |/
            0----------1

        */
        let color = colors::CYAN_COLOR; //Blue bbox
        #[rustfmt::skip] let vertices = [
            LinesVertexData{position: bbox.min, color}, //point0
            LinesVertexData{position: [bbox.max[0], bbox.min[1], bbox.min[2]], color}, //point 1

            LinesVertexData{position: [bbox.max[0], bbox.min[1], bbox.min[2]], color}, //point 1
            LinesVertexData{position: [bbox.max[0], bbox.max[1], bbox.min[2]], color}, //point 2

            LinesVertexData{position: [bbox.max[0], bbox.max[1], bbox.min[2]], color}, //point 2
            LinesVertexData{position: [bbox.min[0], bbox.max[1], bbox.min[2]], color}, //point 3

            LinesVertexData{position: [bbox.min[0], bbox.max[1], bbox.min[2]], color}, //point 3
            LinesVertexData{position: bbox.min, color}, //point0

            LinesVertexData{position: [bbox.min[0], bbox.min[1], bbox.max[2]], color}, //point 4
            LinesVertexData{position: [bbox.max[0], bbox.min[1], bbox.max[2]], color}, //point 5

            LinesVertexData{position: [bbox.max[0], bbox.min[1], bbox.max[2]], color}, //point 5
            LinesVertexData{position: [bbox.max[0], bbox.max[1], bbox.max[2]], color}, //point 6

            LinesVertexData{position: [bbox.max[0], bbox.max[1], bbox.max[2]], color}, //point 6
            LinesVertexData{position: [bbox.min[0], bbox.max[1], bbox.max[2]], color}, //point 7

            LinesVertexData{position: [bbox.min[0], bbox.max[1], bbox.max[2]], color}, //point 7
            LinesVertexData{position: [bbox.min[0], bbox.min[1], bbox.max[2]], color}, //point 4

            LinesVertexData{position: bbox.min, color}, //point0
            LinesVertexData{position: [bbox.min[0], bbox.min[1], bbox.max[2]], color}, //point 4

            LinesVertexData{position: [bbox.max[0], bbox.min[1], bbox.min[2]], color}, //point 1
            LinesVertexData{position: [bbox.max[0], bbox.min[1], bbox.max[2]], color}, //point 5

            LinesVertexData{position: [bbox.max[0], bbox.max[1], bbox.min[2]], color}, //point 2
            LinesVertexData{position: [bbox.max[0], bbox.max[1], bbox.max[2]], color}, //point 6

            LinesVertexData{position: [bbox.min[0], bbox.max[1], bbox.min[2]], color}, //point 3
            LinesVertexData{position: [bbox.min[0], bbox.max[1], bbox.max[2]], color}, //point 7
            ];

        vertices
    }

    fn transform(&self, matrix: &Mat4) -> Self {
        // TODO: fix this
        // transform local bbox and save to global_bounding_box
        let min = self.min;
        let max = self.max;
        let min = Vec4::new(min[0], min[1], min[2], 1.0);
        let max = Vec4::new(max[0], max[1], max[2], 1.0);
        let min = matrix * min;
        let max = matrix * max;

        let min = [min.x, min.y, min.z];
        let max = [max.x, max.y, max.z];

        Self{ min, max }
    }
}

impl BoundingBoxComponent {
    pub fn new(device: &wgpu::Device, bbox: BoundingBox) -> Self {
        let vertices = BoundingBox::gen_vertices(&bbox);

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Dynamic BBox Vertex Buffer"),
            size: (vertices.len() * std::mem::size_of::<LinesVertexData>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            global_bounding_box: bbox.clone(),
            bounding_box: bbox,
            vertex_buffer,
            vertices,
        }
    }
}
