use super::*;
use crate::math::*;

use crate::bounding_box::BoundingBox;

const CORNERS: usize = 8;
type BBoxCornerData = [Vec3; CORNERS];

impl BoundingBoxComponent {
    #[allow(dead_code)]
    pub fn new(bbox: BoundingBox) -> Self {
        Self {
            bounding_box: bbox.clone(),
            global_bounding_box: bbox.clone(),
        }
    }
}

impl BoundingBox {
    pub fn gen_corners(&self) -> BBoxCornerData {
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
