use std::collections::HashSet;

use crate::BoundingBox;
use crate::Camera;
use crate::EntityRawU64;
use crate::app::App;
use crate::ecs::components::BoundingBoxComponent;
use crate::prelude::*;
use legion::Entity;
use legion::EntityStore;

impl App {
    pub fn recenter_camera(&mut self) {
        let camera = &mut self.camera;
        let world = &self.current_scene.world;

        let bbox = if !self.multiselct.is_empty() {
            get_bbox_from_entities(world, &self.multiselct)
        } else if let Some(selected) = self.selected {
            get_bbox_from_entity(world, selected)
        } else {
            get_bounding_box_from_world(world)
        };

        center_camera_to_bounding_box(camera, bbox);

        fn get_bbox_from_entities(
            world: &legion::World,
            entities: &HashSet<u64>,
        ) -> Option<BoundingBox> {
            entities
                .iter()
                .filter_map(|id| {
                    let entity = EntityRawU64::from_raw_u64(*id);
                    let entry = world.entry_ref(entity).ok()?;
                    entry
                        .get_component::<BoundingBoxComponent>()
                        .ok()
                        .map(|b| b.global_bounding_box.clone())
                })
                .reduce(|mut bbox, other| {
                    bbox.merge(&other);
                    bbox
                })
        }

        fn get_bbox_from_entity(world: &legion::World, entity: Entity) -> Option<BoundingBox> {
            let entry = world.entry_ref(entity).ok()?;
            entry
                .get_component::<BoundingBoxComponent>()
                .ok()
                .map(|b| b.global_bounding_box.clone())
        }

        fn get_bounding_box_from_world(world: &legion::World) -> Option<BoundingBox> {
            use legion::IntoQuery;
            <&BoundingBoxComponent>::query()
                .iter(world)
                .map(|b| b.global_bounding_box.clone())
                .reduce(|mut acc, b| {
                    acc.merge(&b);
                    acc
                })
        }

        fn center_camera_to_bounding_box(camera: &mut Camera, bbox: Option<BoundingBox>) {
            if let Some(bbox) = bbox {
                info!("Recenter Camera");
                debug!("Camera {:?}", bbox);
                use crate::math::*;
                let min = Vec3::new(bbox.min[0], bbox.min[1], bbox.min[2]);
                let max = Vec3::new(bbox.max[0], bbox.max[1], bbox.max[2]);
                let size = max - min;
                let fit_offset = 1.1f32;

                let fov = camera.get_fov();
                let aspect = camera.get_aspect();
                let max_size = size.magnitude();
                let fit_height_distance = max_size / Angle::tan(fov);
                let fit_width_distance = fit_height_distance / aspect;

                let distance = fit_offset * fit_height_distance.max(fit_width_distance);
                let center = (min + max) * 0.5;
                let near = distance / 100.0;
                let far = distance * 100.0;

                camera.set_near_far((near, far));
                camera.set_focal_point(center);
                camera.set_distance(distance);
            }
        }
    }
}
