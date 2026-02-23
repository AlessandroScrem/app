use crate::prelude::*;
use crate::input::Input;
use legion::Entity;
use legion::EntityStore;


impl App {

    pub(crate) fn update_camera(&mut self, input: &Input) {
        // move away from here

        if input.is_mouse_button_down(crate::input::MouseButton::Left) {
            let delta = (input.mouse_delta.x as f64, input.mouse_delta.y as f64);
            self.camera.orbit(delta);
        }

        if input.is_mouse_button_down(crate::input::MouseButton::Middle) {
            let delta = (input.mouse_delta.x as f64, input.mouse_delta.y as f64);
            self.camera.pan(delta);
        }

        if let Some(delta) = input.mouse_wheel_movement {
            self.camera.zoom(delta.y);
        }
    }

    pub(crate) fn recenter_camera(&mut self) {
        let camera = &mut self.camera;
        let world = &self.current_scene.world;
        let selected = self.selected;

        let bbox = {
            if let Some(selected) = selected {
                get_bbox_from_entity(world, selected)
            } else {
                get_bounding_box_from_world(world)
            }
        };

        center_camera_to_bounding_box(camera, bbox.clone());

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
                debug!("Recenter Camera {:?}", bbox);
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
