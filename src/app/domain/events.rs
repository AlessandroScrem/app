use crate::assets::gltf_loader::GltfGroup;
use crate::assets::material_desc::MaterialDesc;
use crate::assets::{IblId, MaterialId};
use crate::ecs::components::*;
use crate::math::*;
use std::path::PathBuf;

use legion::Entity;

#[allow(dead_code)]
pub enum DomainEvent {
    Scene(SceneEvent),
    Camera(CameraEvent),
    Global(GlobalEvent),
    Assets(AssetEvent),
    Entity(EntityEvent),
    Selection(SelectionEvent),
}

pub enum SceneEvent {
    Save,
    SaveAs(PathBuf),
    Open(PathBuf),
    AddComponent(GltfGroup, TransformComponent),
    ClearScene,
}

pub enum SelectionEvent {
    Hovered(Option<Entity>),
    Select(Vec<u64>),
    SelectIbl(IblId),
}
pub enum AssetEvent {
    LoadGltf(PathBuf),
    AddIbl(PathBuf),
    #[allow(dead_code)]
    UpdateMaterial(MaterialId, MaterialDesc),
}
pub enum EntityEvent {
    AddLight,
    RemoveEntity(Entity),
    AddParent(Entity),
    #[allow(dead_code)]
    UpdateTag(Entity, TagComponent),
    UpdateTransform(Entity, TransformComponent),
    #[allow(dead_code)]
    UpdateLight(Entity, LightComponent),
    EnableAllLight(bool),
    DisableEntity(Entity, bool),
}
pub enum GlobalEvent {
    LightEnable(bool),
    IblEnable(bool),
    SkyboxEnable(bool),
    SkyboxEnableBlur(bool),
    AxisEnable(bool),
    BboxEnable(bool),
    BboxAxisAligned(bool),
    DebugCode(u32),
    Exposure(f32),
    IblIntensity(f32),
    TonemapFilter(u32),
    MipsCsEnable(bool),
    EnvRotation(f32),
}

pub enum CameraEvent {
    CameraOrbit(f64, f64),
    CameraPan(f64, f64),
    CameraZoom(f32),
    RecenterCamera,
    CameraFov(Rad<f32>),
    CameraDistance(f32),
    CameraNearFar((f32, f32)),
}
