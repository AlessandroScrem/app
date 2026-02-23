
use wgpu::util::DeviceExt;

use crate::assets::vertexdata::LinesVertexData;
use crate::math::*;
use crate::prelude::*;
use crate::{BoundingBoxComponent, colors};


pub(crate) struct BBoxManager {
    vertexbuffer: Option<wgpu::Buffer>,
    count: u32,
}

impl BBoxManager {
    pub(crate) fn new() -> Self {
        Self {
            vertexbuffer: None,
            count: 0,
        }
    }

    pub(crate) fn create_buffer(&mut self, device: &wgpu::Device, vertices: &Vec<BBoxVertexData>) {
        let vertexbuffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some("BBox Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        self.count = (vertices.len() * VERTICES) as u32;
        self.vertexbuffer = Some(vertexbuffer);
    }

    pub(crate) fn get_count(&self)->u32 {
        self.count
    }
    pub(crate) fn get_vertexbuffer(&self) -> &wgpu::Buffer {
        &self.vertexbuffer.as_ref().expect("vb not exist")
    }
}

const VERTICES: usize = 24;
const CORNERS: usize = VERTICES / 3;

pub(crate) type BBoxVertexData = [LinesVertexData; VERTICES];
type BBoxCornerData = [Vec3; CORNERS];

impl BoundingBox {
    fn gen_corners(&self) -> BBoxCornerData {
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

    pub(crate) fn transform_aabb(&self, matrix: &Mat4) -> Self {
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
    pub(crate) fn new(bbox: BoundingBox) -> Self {
        Self {
            bounding_box: bbox.clone(),
            global_bounding_box: bbox.clone(),
        }
    }

    pub(crate) fn gen_aabb_vertices(&self) -> BBoxVertexData {
        let corners = self.global_bounding_box.gen_corners();
        Self::gen_vertices(corners)
    }

    pub(crate) fn gen_obb_vertices(&self, model: &Mat4) -> BBoxVertexData {
        let local_corners = self.bounding_box.gen_corners();

        // Trasforma i corner con la Mat4 dell'oggetto
        let corners = local_corners.map(|c| (model * c.extend(1.0)).truncate());

        Self::gen_vertices(corners)
    }

    fn gen_vertices(corners: BBoxCornerData) -> BBoxVertexData {
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
}
