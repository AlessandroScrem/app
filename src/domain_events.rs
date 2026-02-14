use super::*;
use legion::*;

pub enum DomainEvent {
    Scene(SceneEvent),
    Camera(CameraEvent),
    Global(GlobalEvent),
    Assets(AssetEvent),
    Entity(EntityEvent),
    Selection(SelectionEvent),
}

pub enum SceneEvent {}

pub enum SelectionEvent {
    Selected(Option<Entity>),
}
pub enum AssetEvent {
    LoadGltf(PathBuf),
    ChangeSkybox(PathBuf),

}
pub enum EntityEvent {
    RemoveEntity(Entity),
    AddParent(Entity),
    UpdateTag(Entity, TagComponent),
    UpdateTransform(Entity, TransformComponent),
    UpdateMaterial(Entity, MaterialDesc),
    UpdateLight(Entity, LightComponent),
    
}
pub enum GlobalEvent {
    IblEnable(bool),
    SkyboxEnable(bool),
    AxisEnable(bool),
    BboxEnable(bool),
    BboxAxisAligned(bool),
    DebugCode(u32),
    Exposure(f32),
    IblIntensity(f32),
    TonemapFilter(u32),
}

pub enum CameraEvent {
    RecenterCamera,
    CameraFov(math::Rad<f32>),
    CameraDistance(f32),
    CameraNearFar((f32, f32)),
}

#[derive(Default)]
pub struct DomainEvents {
    pub queue: VecDeque<DomainEvent>,
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

pub fn handle_entity_event(app: &mut App, event: EntityEvent) {
    let world = &mut app.current_scene.world;
    match event {
        EntityEvent::RemoveEntity(entity) => {
            crate::entities::remove_from_root(entity, world);
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