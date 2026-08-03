use crate::app::domain::events::*;
use crate::app::*;
use crate::assets::IblAsset;
use crate::assets::MaterialAsset;
use crate::ecs::components::light;
use crate::ecs::components::*;
use crate::engine::RuntimeEvent;
use crate::engine::engine::EventBus;
use crate::prelude::*;
use crate::scene;

use legion::*;

impl App {
    pub fn update_domain_event(&mut self, bus: &mut EventBus) {
        let mut domain_events = bus.drain_domain();

        while let Some(event) = domain_events.pop() {
            match event {
                DomainEvent::Camera(event) => {
                    handle_camera_event(self, event);
                }
                DomainEvent::Global(event) => {
                    handle_global_event(self, event, bus);
                }
                DomainEvent::Entity(event) => {
                    handle_entity_event(self, event);
                }
                DomainEvent::Assets(event) => {
                    handle_asset_event(self, event, bus);
                }
                DomainEvent::Selection(event) => {
                    handle_selection_event(self, event, bus);
                }
                DomainEvent::Scene(event) => {
                    handle_scene_event(self, event, bus);
                }
            }
        }
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
        CameraEvent::CameraOrbit(dx, dy) => {
            app.camera.orbit((dx, dy));
        }
        CameraEvent::CameraPan(dx, dy) => {
            app.camera.pan((dx, dy));
        }
        CameraEvent::CameraZoom(delta) => {
            app.camera.zoom(delta);
        }
    }
}

pub fn handle_scene_event(app: &mut App, event: SceneEvent, bus: &mut EventBus) {
    match event {
        SceneEvent::ClearScene => {
            app.current_scene.clear_scene(&mut app.asset_mgr);
            app.selected = None;
        }
        SceneEvent::SaveAs(path) => {
            let _ = app.current_scene.save_scene_json(path);
        }
        SceneEvent::Save => {
            let _ = app.current_scene.save();
        }
        SceneEvent::Open(path) => {
            if app.current_scene.open_scene(&path, &mut app.asset_mgr, bus).is_ok() {
                app.current_scene.clear_scene(&mut app.asset_mgr);
                app.selected = None;
                app.settings.add_recent_file(path.into());
            }
        }
        SceneEvent::AddComponent(loaded_scene, transform) => {
            scene::spawn_scene(
                &mut app.current_scene.world,
                &loaded_scene,
                &app.asset_mgr,
                transform,
            );
            bus.send_domain(DomainEvent::Camera(CameraEvent::RecenterCamera));
        }
    }
}

pub fn handle_global_event(app: &mut App, event: GlobalEvent, bus: &mut EventBus) {
    let g = &mut app.globals;
    match event {
        GlobalEvent::LightEnable(flag) => {
            g.light_enable = flag;
            bus.send_domain(DomainEvent::Entity(EntityEvent::EnableAllLight(flag)));
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
            hierarchy::remove_entity(&mut app.asset_mgr, entity, world);
            app.selected = None;
        }
        EntityEvent::AddParent(entity) => {
            hierarchy::add_parent(entity, world);
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
            light::enable_all_lights(enable, world);
        }
        EntityEvent::AddLight => {
            light::create(world);
        }
        EntityEvent::DisableEntity(entity, disable) => {
            hierarchy::disable_entity(entity, world, disable);
        }
    }
}

pub fn handle_asset_event(app: &mut App, event: AssetEvent, bus: &mut EventBus) {
    match event {
        AssetEvent::UpdateMaterial(material_id, desc) => {
            app.asset_mgr.update::<MaterialAsset>(material_id, |asset| {
                asset.desc = desc;
            });
        }
        AssetEvent::LoadGltf(path) => {
            if let Some(loaded) = crate::assets::gltf_loader::load_gltf(path, &mut app.asset_mgr) {
                info!("Loaded: {} Meshes", loaded.meshes.len());
                bus.send_domain(DomainEvent::Scene(SceneEvent::AddComponent(
                    loaded,
                    TransformComponent::default(),
                )));
            }
        }
        AssetEvent::AddIbl(path) => {
            use crate::assets::texture_asset::*;
            let texture_asset = create_texture(path.clone(), TextureUsage::HDR16);
            let hdr_id = app.asset_mgr.add::<TextureAsset>(texture_asset);
            app.asset_mgr.add::<IblAsset>(IblAsset::new(hdr_id, path));
        }
    }
}

pub fn handle_selection_event(app: &mut App, event: SelectionEvent, bus: &mut EventBus) {
    match event {
        SelectionEvent::Hovered(entity) => {
            app.hovered = entity;
        }
        SelectionEvent::Select(entity) => {
            app.selected = entity;
        }
        SelectionEvent::SelectHovered => {
            app.selected = app.hovered;
        }
        SelectionEvent::SelectIbl(ibl_id) => {
            app.selected_ibl = Some(ibl_id);
            bus.send_runtime(RuntimeEvent::UpdateIblMaps(ibl_id));
            println!("Selected {:?}", ibl_id);
        }
        SelectionEvent::SelectionBox(_, _) => {}
    }
}
