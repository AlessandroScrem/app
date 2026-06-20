use crate::{assets::MaterialId, prelude::*};
use crate::assets::material_asset::MaterialAsset;
use std::{collections::VecDeque, path::PathBuf};

use legion::Entity;

#[derive(Default)]
pub struct DomainEvents {
    pub queue: VecDeque<DomainEvent>,
}

#[allow(dead_code)]
pub enum DomainEvent {
    Scene(SceneEvent),
    Camera(CameraEvent),
    Global(GlobalEvent),
    Assets(AssetEvent),
    Entity(EntityEvent),
    Selection(SelectionEvent),
    Exit,
}

pub enum SceneEvent {
    ClearScene,
}

pub enum SelectionEvent {
    Selected(Option<Entity>),
}
pub enum AssetEvent {
    LoadGltf(PathBuf),
    ChangeSkybox(PathBuf),
    UpdateMaterial(MaterialId, MaterialAsset),
}
pub enum EntityEvent {
    RemoveEntity(Entity),
    AddParent(Entity),
    UpdateTag(Entity, TagComponent),
    UpdateTransform(Entity, TransformComponent),
    UpdateLight(Entity, LightComponent),
    EnableAllLight(bool),
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
    RecenterCamera,
    CameraFov(math::Rad<f32>),
    CameraDistance(f32),
    CameraNearFar((f32, f32)),
}
