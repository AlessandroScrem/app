use std::collections::HashMap;

use legion::Entity;

use crate::assets::vertexdata::LinesVertexData;
use crate::math::*;
use crate::prelude::*;
use crate::{BoundingBoxComponent, colors};

pub struct BBoxManager {
    vertexbuffers: HashMap<Entity, wgpu::Buffer>,
}

impl BBoxManager {
    pub fn new() -> Self {
        Self {
            vertexbuffers: HashMap::new(),
        }
    }

    fn create_buffer(&mut self, device: &wgpu::Device, id: Entity) {
        let vertexbuffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Dynamic BBox Vertex Buffer"),
            size: (std::mem::size_of::<BBoxVertexData>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.vertexbuffers.insert(id, vertexbuffer);
    }

    pub fn get_or_create(&mut self, device: &wgpu::Device, id: Entity) -> &wgpu::Buffer {
        if !self.vertexbuffers.contains_key(&id) {
            self.create_buffer(device, id);
            debug!("Create BBOX Buffer");
        }
        self.vertexbuffers.get(&id).expect("vb not exist")
    }

    pub fn get(&self, id: Entity) -> &wgpu::Buffer {
        self.vertexbuffers.get(&id).expect("vb not exist")
    }
}

const VERTICES: usize = 24;
const CORNERS: usize = VERTICES / 3;

pub type BBoxVertexData = [LinesVertexData; VERTICES];
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

    pub fn transform_aabb(&self, matrix: &Mat4) -> Self {
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
    pub fn new(bbox: BoundingBox) -> Self {
        Self {
            bounding_box: bbox.clone(),
            global_bounding_box: bbox.clone(),
        }
    }

    pub fn gen_aabb_vertices(&self) -> BBoxVertexData {
        let corners = self.global_bounding_box.gen_corners();
        Self::gen_vertices(corners)
    }

    pub fn gen_obb_vertices(&self, model: &Mat4) -> BBoxVertexData {
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
