use crate::DomainEvent;
use crate::DomainEvents;
use crate::application_handler::RunningApp;
use crate::assets::asset_manager::AssetManager;
use crate::input::Input;

use crate::Globals;
use crate::prelude::*;
use crate::scene::Scene;

use legion::Entity;
use legion::EntityStore;
use legion::Resources;

#[derive(Default)]
pub struct App {
    pub current_scene: Scene,
    pub asset_mgr: AssetManager,
    pub resources: Resources,
    pub globals: Globals,
    pub camera: Camera,
    pub domain_events: DomainEvents,
    pub selected: Option<Entity>,
    pub hovered: Option<Entity>,
}

impl App {
    


    pub fn update_selected(&mut self, runtime: &mut RunningApp) {
        let input = &runtime.input;
        let renderer = &mut runtime.renderer;
        // update hovered entity_id from buffer
        use crate::input::MouseButton;
        use winit::keyboard::{Key, NamedKey};
        if input.is_cursor_moved() {
            self.hovered = renderer.get_hovered();
        }

        if input.is_mouse_button_pressed(MouseButton::Left)
            && input.is_key_down(Key::Named(NamedKey::Alt))
        {
            self.selected = self.hovered;
        }
    }

    pub fn update_scene(&mut self) {
        self.current_scene
            .schedule
            .execute(&mut self.current_scene.world, &mut self.resources);
    }

    pub fn update_domain_event(&mut self) {
        // event needs world update, will be executed next frame.
        let mut next_queue = VecDeque::<DomainEvent>::new();

        while let Some(event) = self.domain_events.queue.pop_front() {
            match event {
                DomainEvent::Camera(event) => {
                    handle_camera_event(self, event);
                }
                DomainEvent::Global(event) => {
                    handle_global_event(self, event);
                }
                DomainEvent::Entity(event) => {
                    handle_entity_event(self, event);
                }
                DomainEvent::Assets(event) => {
                    handle_asset_event(self, event, &mut next_queue);
                }
                DomainEvent::Selection(event) => {
                    handle_selection_event(self, event);
                }
            }
        }

        self.domain_events.queue.append(&mut next_queue);
    }

    pub fn update_camera(&mut self, input: &Input) {
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


    pub fn update_uilayer(&mut self, runtime: &mut RunningApp) {
        let uilayer = &mut runtime.uilayer;
        let renderer = &mut runtime.renderer;
        let window = &runtime.window;

        let snapshot = UiSnapshot::from_world(
            &self.current_scene.world,
            self.selected,
            &self.asset_mgr,
            &self.camera,
            &self.globals,
            renderer,
            None,
        );

        // Main operation: update_ui
        let mut events = uilayer.build(window, snapshot);
        self.domain_events.queue.append(&mut events);
    }

    pub fn recenter_camera(&mut self) {
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
