use std::sync::Arc;

use crate::{
    BoundingBoxComponent, Globals, MeshComponent, TransformComponent, colors,
    renderer::{
        gpu_manager::GPUResourceManager,
        hdr_frame::HdrFrame,
        pipeline_manager::{PipelineKind, PipelineManager},
    },
};

use cgmath::vec4;
use legion::{world::SubWorld, *};

#[system]
#[read_component(BoundingBoxComponent)]
pub fn bounding_box(
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
#[filter(maybe_changed::<TransformComponent>())]
pub fn update_bounding_box(
    transform: &TransformComponent,
    bbox_component: &mut BoundingBoxComponent,
    mesh: &MeshComponent,
    #[resource] queue: &wgpu::Queue,
    #[resource] globals: &Globals,
) {
    if !globals.bbox_enable {
        return;
    }

    println!("BoundingBox maybe_changed");

    let bounding_box = BoundingBox {
        min: mesh.data.vmin,
        max: mesh.data.vmax,
    };

    let vertices = BoundingBox::gen_vertices(&bounding_box, &transform);

    queue.write_buffer(
        &bbox_component.vertex_buffer,
        0,
        bytemuck::cast_slice(&vertices.as_slice()),
    );
}

use crate::{assets::vertexdata::LinesVertexData, entities::bounding_box::BoundingBox};

impl BoundingBox {
    pub fn transform(&mut self, transform: &TransformComponent) {
        let matrix = transform.compute_model_matrix();

        let min = vec4(self.min[0], self.min[1], self.min[2], 1.0);
        let max = vec4(self.max[0], self.max[1], self.max[2], 1.0);
        let tmin: cgmath::Vector4<f32> = matrix * min;
        let tmax: cgmath::Vector4<f32> = matrix * max;

        self.min = [tmin.x, tmin.y, tmin.z];
        self.max = [tmax.x, tmax.y, tmax.z];
    }
}

impl BoundingBox {
    pub fn gen_vertices(
        bbox: &BoundingBox,
        transform: &TransformComponent,
    ) -> [LinesVertexData; 24] {
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
        #[rustfmt::skip] let mut vertices = [
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

        let matrix = transform.compute_model_matrix();
        vertices.iter_mut().for_each(|v| {
            let pos = cgmath::Vector4::new(v.position[0], v.position[1], v.position[2], 1.0);
            let t = matrix * pos;
            v.position = [t.x, t.y, t.z];
        });

        vertices
    }

    pub fn create_vertex_buffer(device: &wgpu::Device, bbox: &BoundingBox, transform: &TransformComponent) -> wgpu::Buffer {
        let vertices = BoundingBox::gen_vertices(bbox, transform);

        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Dynamic BBox Vertex Buffer"),
            size: (vertices.len() * std::mem::size_of::<LinesVertexData>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }
}
