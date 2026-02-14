use std::collections::VecDeque;
use legion::*;
use crate::engine::RunningApp;
use crate::prelude::*;
use crate::App;


impl App {
       pub fn update_domain_event(&mut self, runtime: &mut RunningApp) {
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
                    handle_entity_event(self, runtime, event);
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
}


pub fn handle_camera_event(app: &mut App, event: CameraEvent) {
    match event {
        CameraEvent::RecenterCamera => {
            app.recenter_camera();
        }
        CameraEvent::CameraFov(fov) => {
            app.camera.set_fov(fov);
        }
        CameraEvent::CameraDistance(distance) => {
            app.camera.set_distance(distance);
        }
        CameraEvent::CameraNearFar(near_far) => {
            app.camera.set_near_far(near_far);
        }
    }
}

pub fn handle_global_event(app: &mut App, event: GlobalEvent) {
    let g = &mut app.globals;
    match event {
        GlobalEvent::IblEnable(flag) => g.ibl_enable = flag,
        GlobalEvent::SkyboxEnable(flag) => g.skybox_enable = flag,
        GlobalEvent::AxisEnable(flag) => g.axis_enable = flag,
        GlobalEvent::BboxEnable(flag) => g.bbox_enable = flag,
        GlobalEvent::BboxAxisAligned(flag) => g.bbox_axis_aligned = flag,
        GlobalEvent::DebugCode(code) => g.debug_code = code,
        GlobalEvent::Exposure(value) => g.exposure = value,
        GlobalEvent::IblIntensity(value) => g.ibl_intensity = value,
        GlobalEvent::TonemapFilter(filter_code) => g.tonemap_filter = filter_code,
    }
}

pub fn handle_entity_event(app: &mut App, runtime: &mut RunningApp, event: EntityEvent) {
    let world = &mut app.current_scene.world;
    match event {
        EntityEvent::RemoveEntity(entity) => {
            crate::entities::remove_entity_from_all(&mut app.asset_mgr, runtime, entity, world);
            app.selected = None;
        }
        EntityEvent::AddParent(entity) => {
            crate::entities::add_parent(entity, world);
        }
        EntityEvent::UpdateTag(entity, c) => {
            if let Ok(mut e) = app.current_scene.world.entry_mut(entity) {
                if let Ok(t) = e.get_component_mut::<TagComponent>() {
                    *t = c;
                }
            }
        }
        EntityEvent::UpdateTransform(entity, c) => {
            if let Ok(mut e) = world.entry_mut(entity) {
                if let Ok(t) = e.get_component_mut::<TransformComponent>() {
                    *t = c;
                }
            }
        }
        EntityEvent::UpdateMaterial(_entity, c) => {
            app.asset_mgr.materials.update(&c);
        }
        EntityEvent::UpdateLight(entity, c) => {
            if let Ok(mut e) = world.entry_mut(entity) {
                if let Ok(light) = e.get_component_mut::<LightComponent>() {
                    *light = c;
                }
            }
        }
    }
}

pub fn handle_asset_event(app: &mut App, event: AssetEvent, next_queue: &mut VecDeque<DomainEvent>) {
    match event {
        AssetEvent::LoadGltf(path) => {
            if let Ok(loaded) = crate::assets::gltf_loader::load_gltf(path, &mut app.asset_mgr) {
                crate::assets::gltf_loader::spawn_scene(
                    &mut app.current_scene.world,
                    &loaded,
                    &app.asset_mgr,
                );
                next_queue.push_back(DomainEvent::Camera(CameraEvent::RecenterCamera));
            }
        }
        AssetEvent::ChangeSkybox(path) => {
            let hdr_id = app
                .asset_mgr
                .textures
                .from_file(path, crate::assets::TextureUsage::HDR16);
            app.asset_mgr.skybox.set_id(hdr_id);
        }
    }
}

pub fn handle_selection_event(app: &mut App, event: SelectionEvent) {
    match event {
        SelectionEvent::Selected(selected) => {
            app.selected = selected;
        }
    }
}