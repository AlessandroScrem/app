#[derive(Debug, Clone)]
pub(crate) struct BoundingBox {
    pub(crate) min: [f32; 3],
    pub(crate) max: [f32; 3],
}

impl Default for BoundingBox{
    fn default() -> Self {
        Self::new_empty()
    }
}

impl BoundingBox {
    pub(crate) fn new_empty() -> Self {
        Self {
            min: [f32::INFINITY; 3],
            max: [f32::NEG_INFINITY; 3],
        }
    }

    pub(crate) fn extend(&mut self, point: &[f32; 3]) {
        for ((mi, ma), &val) in self.min.iter_mut().zip(self.max.iter_mut()).zip(point) {
            *mi = mi.min(val);
            *ma = ma.max(val);
        }
    }

    pub(crate) fn merge(&mut self, other: &BoundingBox) {
        self.extend(&other.min);
        self.extend(&other.max);
    }

    #[allow(dead_code)]
    pub(crate) fn from_points<'a, I: IntoIterator<Item = &'a [f32; 3]>>(points: I) -> Self {
        let mut bbox = BoundingBox::new_empty();
        for p in points {
            bbox.extend(p);
        }
        bbox
    }
}

// Implementazione di From per tuple di array ([f32;3], [f32;3])
impl From<([f32; 3], [f32; 3])> for BoundingBox {
    fn from(value: ([f32; 3], [f32; 3])) -> Self {
        BoundingBox {
            min: value.0,
            max: value.1,
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_bounding_box_from_points() {
        let points: Vec<[f32; 3]> = vec![
            [1.0, 5.0, 3.0],  //
            [2.0, -1.0, 7.0], //
            [0.5, 4.0, 6.0],
        ];

        let aabb = BoundingBox::from_points(points.iter());

        assert_eq!(aabb.min, [0.5, -1.0, 3.0]);
        assert_eq!(aabb.max, [2.0, 5.0, 7.0]);
    }

    #[test]
    fn should_bounding_box_from_single_point() {
        let points = vec![[1.0, 2.0, 3.0]];

        let aabb = BoundingBox::from_points(points.iter());

        assert_eq!(aabb.min, [1.0, 2.0, 3.0]);
        assert_eq!(aabb.max, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_aabb_empty() {
        let bbox = BoundingBox::new_empty();

        // min rimane INFINITY, max rimane NEG_INFINITY
        assert_eq!(bbox.min, [f32::INFINITY; 3]);
        assert_eq!(bbox.max, [f32::NEG_INFINITY; 3]);
    }

    #[test]
    fn test_extend_step_by_step() {
        let mut bbox = BoundingBox::new_empty();

        // dopo il primo punto
        bbox.extend(&[1.0, 2.0, 3.0]);
        assert_eq!(bbox.min, [1.0, 2.0, 3.0]);
        assert_eq!(bbox.max, [1.0, 2.0, 3.0]);

        // aggiungo un punto "più piccolo"
        bbox.extend(&[0.0, -1.0, 2.5]);
        assert_eq!(bbox.min, [0.0, -1.0, 2.5]);
        assert_eq!(bbox.max, [1.0, 2.0, 3.0]);

        // aggiungo un punto "più grande"
        bbox.extend(&[5.0, 3.0, 10.0]);
        assert_eq!(bbox.min, [0.0, -1.0, 2.5]);
        assert_eq!(bbox.max, [5.0, 3.0, 10.0]);
    }
}
