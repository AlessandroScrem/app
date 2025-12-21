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

    bbox_component.global_bounding_box = bbox_component
        .bounding_box
        .transform_aabb(&global_model.mat);

    let vertices = {
        if globals.bbox_axis_aligned {
            BoundingBox::gen_aabb_vertices(&bbox_component.global_bounding_box)
        } else {
            BoundingBox::gen_obb_vertices(&bbox_component.bounding_box, &global_model.mat)
        }
    };

    queue.write_buffer(
        &bbox_component.vertex_buffer,
        0,
        bytemuck::cast_slice(&vertices.as_slice()),
    );
}

const VERTICES: usize = 24;
const CORNERS: usize = VERTICES / 3;

impl BoundingBox {
    pub fn gen_aabb_vertices(bbox: &BoundingBox) -> [LinesVertexData; VERTICES] {
        let corners = bbox.gen_corners();
        Self::gen_vertices(corners)
    }

    pub fn gen_obb_vertices(bbox: &BoundingBox, model: &Mat4) -> [LinesVertexData; VERTICES] {
        let local_corners = bbox.gen_corners();

        // Trasforma i corner con la Mat4 dell'oggetto
        let corners = local_corners.map(|c| (model * c.extend(1.0)).truncate());

        Self::gen_vertices(corners)
    }

    fn gen_vertices(corners: [Vec3; CORNERS]) -> [LinesVertexData; VERTICES] {
        let edges = [
            // bottom
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 0),
            // top
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 4),
            // vertical
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7),
        ];

        let color = colors::CYAN_COLOR;
        let mut vertices = [LinesVertexData::default(); VERTICES];
        for (i, &(a, b)) in edges.iter().enumerate() {
            let base = i * 2;
            vertices[base] = LinesVertexData {
                position: corners[a].into(),
                color,
            };
            vertices[base + 1] = LinesVertexData {
                position: corners[b].into(),
                color,
            }
        }
        vertices
    }

    fn gen_corners(&self) -> [Vec3; CORNERS] {
        /*
        bbox vertices order:
            y  7----------6
            | /|         /|
            |/ |        / |
            3----------2  |
            |  | z     |  |
            |  4-------|--5
            | /        | /
            |/         |/
            0----------1 --->x
        */
        [
            Vec3::new(self.min[0], self.min[1], self.min[2]),
            Vec3::new(self.max[0], self.min[1], self.min[2]),
            Vec3::new(self.max[0], self.max[1], self.min[2]),
            Vec3::new(self.min[0], self.max[1], self.min[2]),
            Vec3::new(self.min[0], self.min[1], self.max[2]),
            Vec3::new(self.max[0], self.min[1], self.max[2]),
            Vec3::new(self.max[0], self.max[1], self.max[2]),
            Vec3::new(self.min[0], self.max[1], self.max[2]),
        ]
    }

    fn transform_aabb(&self, matrix: &Mat4) -> Self {
        let corners = self.gen_corners();

        // Trasformazione
        let transformed = corners.map(|c| matrix * c.extend(1.0));

        // Ricostruzione AABB
        let mut bbox = Self::new_empty();
        for p in transformed {
            bbox.extend(&p.truncate().into());
        }
        bbox
    }
}

impl BoundingBoxComponent {
    pub fn new(device: &wgpu::Device, bbox: BoundingBox) -> Self {
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Dynamic BBox Vertex Buffer"),
            size: (VERTICES * std::mem::size_of::<LinesVertexData>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            global_bounding_box: bbox.clone(),
            bounding_box: bbox,
            vertex_buffer,
        }
    }
}
