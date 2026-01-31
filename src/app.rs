use std::collections::HashMap;

use crate::BoundingBoxComponent;
use crate::DomainEvent;
use crate::DomainEvents;
use crate::HierarchyComponent;
use crate::LightComponent;
use crate::MeshComponent;
use crate::TagComponent;
use crate::TransformComponent;
use crate::UiComponentView;
use crate::application_handler::RunningApp;
use crate::assets::asset_manager::AssetManager;
use crate::input::Input;
use crate::prelude::ui::ui_layer::HierarchyNode;
use crate::prelude::ui::ui_layer::RootNodes;
use crate::prelude::ui::ui_layer::RootSnapshot;
use crate::prelude::ui::ui_layer::Snapshot;

use crate::Globals;
use crate::prelude::*;
use crate::renderer;
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
    pub fn init(&mut self) {
        let timer = std::time::Instant::now();

        self.domain_events.queue.push_back(DomainEvent::LoadGltf(
            "./assets/Lantern/Lantern.gltf".into(),
        ));

        crate::entities::light::create(&mut self.current_scene.world, &self.resources);

        self.current_scene.schedule = crate::systems::create_current_scene_schedule_builder();

        debug!("App loader took {} ms", timer.elapsed().as_millis());
    }

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

    pub fn render(&mut self, runtime: &mut RunningApp) {
        runtime.renderer.render(
            &self.asset_mgr,
            &self.current_scene.world,
            &mut self.resources,
            &self.camera,
            &self.globals,
            self.selected,
            &runtime.input,
            runtime.uilayer.get_draw_data(),
        );
    }

    pub fn update_uilayer(&mut self, runtime: &mut RunningApp) {
        let uilayer = &mut runtime.uilayer;
        let renderer = &mut runtime.renderer;
        let window = &runtime.window;

        let root_snapshot = create_root_snapshot(&self.current_scene.world);
        let comp_view = &mut get_comp_view(
            self.selected,
            &self.current_scene.world,
            &self.asset_mgr,
            &renderer.get_texture_registry(),
        );

        let mut snapshot = Snapshot {
            camera: &mut self.camera,
            globals: &mut self.globals,
            root_nodes: &root_snapshot.root_nodes,
            lights_nodes: &root_snapshot.lights_nodes,
            comp_view,
            selected: &mut self.selected,
            hovered: self.hovered,
            adapter_string: renderer.get_adapter_string(),
            hdr_texture_id: renderer.get_hdr_imgui_id(),
            debug_texture_id: None,
        };

        // Main operation: update_ui
        let mut events = uilayer.build(window, &mut snapshot);
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

                let fov = camera.fov;
                let aspect = camera.get_aspect();
                let max_size = size.magnitude();
                let fit_height_distance = max_size / Angle::tan(fov);
                let fit_width_distance = fit_height_distance / aspect;

                let distance = fit_offset * fit_height_distance.max(fit_width_distance);
                let center = (min + max) * 0.5;
                let near = distance / 100.0;
                let far = distance * 100.0;

                camera.near = near;
                camera.far = far;
                camera.set_focal_point(center);
                camera.set_distance(distance);
            }
        }
    }
}

fn create_root_snapshot(world: &legion::World) -> RootSnapshot {
    let root_nodes = get_hierarchy_roots(world);
    let lights_nodes = get_lights_roots(world);

    RootSnapshot {
        root_nodes,
        lights_nodes,
    }
}

use legion::query::IntoQuery;
fn get_lights_roots(world: &legion::World) -> RootNodes {
    let mut roots = RootNodes::default();
    let mut query = <(Entity, &LightComponent, &TagComponent)>::query();
    for (entity, _light, tag) in query.iter(world) {
        let name = tag.name.clone();
        let node = HierarchyNode {
            name,
            parent: None,
            entity: entity.clone(),
            children: Vec::new(),
        };

        roots.nodes.push(node);
    }
    roots
}

fn get_hierarchy_roots(world: &legion::World) -> RootNodes {
    let mut query = <(Entity, &HierarchyComponent)>::query();
    let mut roots = RootNodes::default();

    for (entity, hierarchy) in query.iter(world) {
        if hierarchy.parent.is_none() {
            let node = build_node(world, *entity, None);
            roots.nodes.push(node);
        }
    }

    fn build_node(world: &legion::World, entity: Entity, parent: Option<Entity>) -> HierarchyNode {
        let entry = world
            .entry_ref(entity)
            .expect(format!("entity {:?} not found", entity).as_str());

        let name = entry
            .get_component::<TagComponent>()
            .map(|n| n.name.clone())
            .unwrap_or("<unnamed>".to_string());

        let hierarchy = entry.get_component::<HierarchyComponent>().unwrap();

        let children = hierarchy
            .children
            .iter()
            .map(|&child| build_node(world, child, Some(entity)))
            .collect();

        HierarchyNode {
            name,
            parent,
            entity,
            children,
        }
    }

    roots
}

fn get_comp_view(
    selected: Option<Entity>,
    world: &legion::World,
    asset_mgr: &AssetManager,
    tex_registry: &renderer::ImGuiTextureRegistry,
) -> UiComponentView {
    let mut comp_view = UiComponentView::default();

    if let Some(selected) = selected {
        if let Ok(entry) = world.entry_ref(selected) {
            if let Ok(light) = entry.get_component::<LightComponent>() {
                comp_view.light = Some(light.clone());
            }
        }
        if let Ok(entry) = world.entry_ref(selected) {
            if let Ok(tag) = entry.get_component::<TagComponent>() {
                comp_view.tag = Some(tag.clone());
            }
        }
        if let Ok(entry) = world.entry_ref(selected) {
            if let Ok(transform) = entry.get_component::<TransformComponent>() {
                comp_view.transform = Some(transform.clone());
            }
        }
        if let Ok(entry) = world.entry_ref(selected) {
            if let Ok(bbox) = entry.get_component::<BoundingBoxComponent>() {
                comp_view.bounding_box = Some(bbox.clone());
            }
        }
        if let Ok(entry) = world.entry_ref(selected) {
            if let Ok(mesh) = entry.get_component::<MeshComponent>() {
                comp_view.mesh = Some(mesh.clone());

                let mut ids = HashMap::new();
                if let Some(mesh_desc) = asset_mgr.meshes.get(mesh.handle) {
                    for submesh in mesh_desc.submeshes.iter() {
                        if let Some(mat_desc) = asset_mgr.materials.get(submesh.material) {
                            for slot in material_manager::MATERIAL_TEXTURE_SLOTS {
                                if let Some(id) = mat_desc.get_texture_slot(slot) {
                                    if let Some(reg_id) = tex_registry.ids.get(&id) {
                                        ids.insert(id.clone(), reg_id.clone());
                                    } 
                                }
                            }
                        }
                    }
                }
                comp_view.texture_id_map = ids;
            }
        }
    }
    comp_view
}
