use crate::app::domain::SceneEvent;
use crate::app::*;
use crate::assets::ResourceStats;
use crate::gpu::ibl_asset::IblAsset;
use crate::gpu::material_asset::MaterialAsset;
use crate::gpu::texture_asset::TextureAsset;
use crate::prelude::*;
use legion::*;
use std::collections::VecDeque;

impl App {
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
                DomainEvent::Scene(event) => {
                    handle_scene_event(self, event);
                }
                DomainEvent::Exit => {
                    self.exit_requested = true;
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

pub fn handle_scene_event(app: &mut App, event: SceneEvent) {
    match event {
        SceneEvent::ClearScene => {
            let world = &mut app.current_scene.world;

            for entity in entities::collect_hierarchy_root_entities(world).iter() {
                crate::entities::remove_entity_from_all(&mut app.asset_mgr, *entity, world);
            }
            app.selected = None;
        }
    }
}

pub fn handle_global_event(app: &mut App, event: GlobalEvent) {
    let g = &mut app.globals;
    match event {
        GlobalEvent::LightEnable(flag) => {
            g.light_enable = flag;
            app.domain_events
                .queue
                .push_back(DomainEvent::Entity(EntityEvent::EnableAllLight(flag)));
        }
        GlobalEvent::IblEnable(flag) => g.ibl_enable = flag,
        GlobalEvent::SkyboxEnable(flag) => g.skybox_enable = flag,
        GlobalEvent::SkyboxEnableBlur(flag) => g.skybox_enable_blur = flag,
        GlobalEvent::AxisEnable(flag) => g.axis_enable = flag,
        GlobalEvent::BboxEnable(flag) => g.bbox_enable = flag,
        GlobalEvent::BboxAxisAligned(flag) => g.bbox_axis_aligned = flag,
        GlobalEvent::DebugCode(code) => g.debug_code = code,
        GlobalEvent::Exposure(value) => g.exposure = value,
        GlobalEvent::IblIntensity(value) => g.ibl_intensity = value,
        GlobalEvent::TonemapFilter(filter_code) => g.tonemap_filter = filter_code,
        GlobalEvent::MipsCsEnable(flag) => g.mips_cs = flag,
        GlobalEvent::EnvRotation(value) => g.env_rotation = value,
    }
}

pub fn handle_entity_event(app: &mut App, event: EntityEvent) {
    let world = &mut app.current_scene.world;
    match event {
        EntityEvent::RemoveEntity(entity) => {
            crate::entities::remove_entity_from_all(&mut app.asset_mgr, entity, world);
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
        EntityEvent::UpdateLight(entity, c) => {
            if let Ok(mut e) = world.entry_mut(entity) {
                if let Ok(light) = e.get_component_mut::<LightComponent>() {
                    *light = c;
                }
            }
        }
        EntityEvent::EnableAllLight(enable) => {
            crate::entities::enable_all_lights(enable, world);
        }
    }
}

pub fn handle_asset_event(
    app: &mut App,
    event: AssetEvent,
    next_queue: &mut VecDeque<DomainEvent>,
) {
    match event {
        AssetEvent::UpdateMaterial(material_id, asset) => {
            app.asset_mgr.update::<MaterialAsset>(material_id, asset);
        }
        AssetEvent::LoadGltf(path) => {
            if let Some(loaded) = crate::assets::gltf_loader::load_gltf(path, &mut app.asset_mgr) {
                info!("Loaded: {} Meshes", loaded.meshes.len());
                entities::spawn_scene(&mut app.current_scene.world, &loaded, &app.asset_mgr);
                next_queue.push_back(DomainEvent::Camera(CameraEvent::RecenterCamera));
            }
        }
        AssetEvent::ChangeSkybox(path) => {
            use crate::assets::texture_asset::TextureUsage;
            let texture_asset =
                assets::texture_asset::create_texture(path.clone(), TextureUsage::HDR16);
            let hdr_id = app.asset_mgr.add::<TextureAsset>(texture_asset);

            if let Some(id) = app.ibl_id {
                let asset = IblAsset::new(hdr_id, path);
                app.asset_mgr.update::<IblAsset>(id, asset)
            }
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
