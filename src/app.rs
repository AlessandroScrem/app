use crate::BoundingBoxComponent;
use crate::DomainEvent;
use crate::DomainEvents;
use crate::GlobalModelComponent;
use crate::HierarchyComponent;
use crate::LightComponent;
use crate::MeshComponent;
use crate::TagComponent;
use crate::TransformComponent;
use crate::UiComponentView;
use crate::assets::material_manager::MaterialManager;
use crate::entities::bounding_box::BoundingBox;
use crate::input::Input;
use crate::prelude::ui::ImguiLayer;
use crate::prelude::ui::state::HierarchyNode;
use crate::prelude::ui::state::RootNodes;
use crate::prelude::ui::state::RootSnapshot;
use crate::prelude::ui::state::Snapshot;
use crate::renderer::gpu_renderer::GpuBoxFrame;
use crate::renderer::gpu_renderer::GpuMeshFrame;
use crate::renderer::gpu_renderer::RenderFrame;
use crate::renderer::renderpass::axis::AxisRenderPass;
use crate::renderer::renderpass::bbox::BboxRenderPass;
use crate::renderer::renderpass::imgui::ImguiRenderPass;
use crate::renderer::renderpass::light::LightRenderPass;
use crate::renderer::renderpass::linearize::LinerizeRenderPass;
use crate::renderer::renderpass::mesh::MeshRenderPass;
use crate::renderer::renderpass::outline::OutlineRenderPass;
use crate::renderer::renderpass::pickobject::PickObjectRenderPass;
use crate::renderer::renderpass::skybox::SkyboxRenderPass;
use crate::renderer::uniform::ModelUniform;
use std::time::Duration;
use std::time::Instant;

use crate::Globals;
use crate::prelude::*;
use crate::scene::Scene;

use legion::Entity;
use legion::EntityStore;
use legion::Resources;

pub struct AppTimer {
    pub clock: Instant,    // timer since application start
    pub delta_time: f32,   //time since last frame
    pub elapsed_time: f32, //timer since last update
    pub frame_time: f32,   //time taken to render last frame
    last_trigger: Instant, // last time the every() callback was triggered
}

impl AppTimer {
    pub const FIXED_TIMESTEP: f32 = 1.0 / 60.0; //minimum timestep (to avoid leg)
    pub fn new() -> Self {
        Self {
            clock: Instant::now(),
            delta_time: 0.0,
            elapsed_time: 0.0,
            frame_time: 0.0,
            last_trigger: Instant::now(),
        }
    }

    /// Returns the time in seconds since the last call to frametime() and updates the internal timer.
    pub fn frametime(&mut self) -> f32 {
        let frametime = self.clock.elapsed().as_secs_f32() - self.elapsed_time;
        self.frame_time = frametime * 1000.0;
        frametime
    }

    /// Update the timer with the given frametime.
    /// Clamps the delta_time to FIXED_TIMESTEP to avoid large timesteps.
    pub fn tick(&mut self, frametime: f32) -> f32 {
        self.delta_time = f32::min(frametime, Self::FIXED_TIMESTEP);
        self.elapsed_time += self.delta_time;
        self.delta_time
    }

    /// Iterator that yields fixed timestep steps until the current frametime is covered.
    /// This can be used to run fixed timestep updates in a variable timestep environment.
    ///
    /// # Example
    /// ```ignore
    /// let mut timer = super::Timer::new();
    /// let frametime = timer.frametime();
    /// for dt in timer.tick_step_iter() {
    ///     // Run fixed timestep update with dt
    /// }
    /// ```
    pub fn tick_step_iter(&mut self) -> impl Iterator<Item = f32> + '_ {
        let mut remaining = self.frametime();
        std::iter::from_fn(move || {
            if remaining > 0.0 {
                let dt = remaining.min(Self::FIXED_TIMESTEP);
                remaining -= dt;
                Some(self.tick(dt))
            } else {
                None
            }
        })
    }

    /// Trigger a callback every `interval` duration.
    /// The callback is called once for each interval that has passed since the last trigger.
    /// This function should be called every frame, typically in the main loop.
    /// # Example
    /// ```ignore
    /// use std::time::Duration;
    /// let mut timer = Timer::new();
    /// loop {
    ///   timer.trigger_every(Duration::from_secs(1), || {
    ///     println!("This prints every second");
    /// });
    /// }
    /// ```
    ///  
    pub fn trigger_every<F>(&mut self, interval: Duration, mut callback: F)
    where
        F: FnMut(),
    {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_trigger);

        if elapsed >= interval {
            // Calcola quanti intervalli completi sono passati
            let steps = elapsed.as_nanos() / interval.as_nanos();
            self.last_trigger += interval * steps as u32;

            // Esegui la callback
            callback();
        }
    }
}
#[derive(Default)]
pub struct App {
    pub current_scene: Scene,
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

    pub fn update_selected(&mut self, input: &Input, renderer: &mut Renderer) {
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

    pub fn render(&mut self, renderer: &mut Renderer, imgui: &mut ImguiLayer, input: &Input) {
        // update gpu data (uniform,  buffers)
        let render_frame = self.extract_render_data(self.selected);
        renderer.prepare(&render_frame);

        let frame = renderer.get_frame();
        let view = frame.texture.create_view(&Default::default());

        // Begin Pass
        let mut encoder = renderer.get_encoder();

        // HDR pass
        MeshRenderPass::new(renderer.get_gpu_view(), &mut encoder).render(&render_frame.meshes);
        LightRenderPass::new(renderer.get_gpu_view(), &mut encoder).render(&render_frame.lights);
        SkyboxRenderPass::new(renderer.get_gpu_view(), &mut encoder)
            .render(self.globals.skybox_enable);
        AxisRenderPass::new(renderer.get_gpu_view(), &mut encoder).render(self.globals.axis_enable);
        BboxRenderPass::new(renderer.get_gpu_view(), &mut encoder)
            .render(&render_frame.bboxes, self.globals.bbox_enable);

        // Hdr to Linear
        LinerizeRenderPass::new(renderer.get_gpu_view(), &mut encoder).render(&view);

        // Ldr pass
        OutlineRenderPass::new(renderer.get_gpu_view(), &mut encoder)
            .render(&view, self.selected.is_some());
        PickObjectRenderPass::new(renderer.get_gpu_view(), &mut encoder).render(&input);
        ImguiRenderPass::new(renderer.get_gpu_view(), &mut encoder).render(&view, imgui);

        // End pass
        renderer.queue.submit([encoder.finish()]);

        frame.present();
    }

    fn extract_render_data(&self, selected: Option<Entity>) -> RenderFrame {
        let world = &self.current_scene.world;
        use crate::entities::EntityRawU64;
        use legion::query::IntoQuery;

        let entity_selected_id = match selected {
            Some(id) => (&id).as_raw_u64(),
            None => 0,
        };

        let mut frame = RenderFrame {
            meshes: Vec::new(),
            lights: Vec::new(),
            bboxes: Vec::new(),
            globals: self.globals,
            camera: self.camera.clone(),
            entity_id: entity_selected_id,
        };

        {
            // -------- Mesh --------
            let mut mesh_query = <(Entity, &MeshComponent, &GlobalModelComponent)>::query();

            for (entity, mesh, global) in mesh_query.iter(world) {
                let mut model = ModelUniform::new(global.mat);
                model.entity_id = entity.as_raw_u64();
                frame.meshes.push(GpuMeshFrame {
                    mesh_handle: mesh.handle,
                    model,
                    material_id: mesh.mat_handle.clone(),
                });
            }
        }

        {
            // -------- Lights --------
            let mut light_query = <&LightComponent>::query();

            for light in light_query.iter(world) {
                frame.lights.push(light.data);
            }
        }

        {
            // -------- BoundingBox --------
            let mut bbox_query = <(Entity, &BoundingBoxComponent, &GlobalModelComponent)>::query();

            for (entity, bbox, global_model) in bbox_query.iter(world) {
                let vertices = {
                    if self.globals.bbox_axis_aligned {
                        bbox.gen_aabb_vertices()
                    } else {
                        bbox.gen_obb_vertices(&global_model.mat)
                    }
                };
                frame.bboxes.push(GpuBoxFrame {
                    vertices,
                    entity: *entity,
                });
            }
        }

        frame
    }

    pub fn imgui_update(
        &mut self,
        imgui: &mut ImguiLayer,
        window: &winit::window::Window,
        renderer: &mut Renderer,
    ) {
        let root_snapshot = create_root_snapshot(&self.current_scene.world);
        let comp_view = &mut get_comp_view(
            self.selected,
            &self.current_scene.world,
            &renderer.get_mat_mgr(),
        );

        let mut snapshot = Snapshot {
            camera: &mut self.camera,
            globals: &mut self.globals,
            root_nodes: &root_snapshot.root_nodes,
            lights_nodes: &root_snapshot.lights_nodes,
            comp_view,
            selected: &mut self.selected,
            hovered: self.hovered,
        };

        let mut events = { imgui.update_ui(window, &mut snapshot) };

        while let Some(event) = events.pop_front() {
            self.domain_events.queue.push_back(event);
        }

        flush_selected(
            &mut self.current_scene.world,
            self.selected,
            comp_view,
            renderer.get_mat_mgr_mut(),
        );

        fn flush_selected(
            world: &mut legion::World,
            selected: Option<Entity>,
            comp_view: &mut UiComponentView,
            mat_mgr: &mut MaterialManager,
        ) {
            if let Some(entity) = selected
                && comp_view.dirty
            {
                println!("Dirty");
                if let Ok(mut entry) = world.entry_mut(entity) {
                    if let Ok(tag) = entry.get_component_mut::<TagComponent>() {
                        comp_view.tag.as_ref().map(|t| *tag = t.clone());
                    }
                    if let Ok(transform) = entry.get_component_mut::<TransformComponent>() {
                        comp_view.transform.as_ref().map(|t| *transform = t.clone());
                    }
                    if let Ok(light) = entry.get_component_mut::<LightComponent>() {
                        comp_view.light.as_ref().map(|t| *light = t.clone());
                    }

                    if let Some(updated_material) = comp_view.material.clone() {
                        if let Ok(mesh) = entry.get_component_mut::<MeshComponent>() {
                            let material = &mut mat_mgr.get_mut(&mesh.mat_handle).material_pbr;
                            *material = updated_material;
                        }
                    }
                }
                comp_view.dirty = false
            }
        }
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

        center_camera_to_bounding_box(camera, bbox);

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

        fn center_camera_to_bounding_box(
            camera: &mut Camera,
            bbox: Option<crate::entities::bounding_box::BoundingBox>,
        ) {
            if let Some(bbox) = bbox {
                println!("Recenter Camera {:?}", bbox);
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
    mat_mgr: &MaterialManager,
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
                comp_view.material = Some(mat_mgr.get(&mesh.mat_handle).material_pbr.clone());
            }
        }
    }
    comp_view
}
