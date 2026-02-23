use std::{collections::VecDeque, path::PathBuf};
use crate::prelude::*;

use legion::Entity;

#[derive(Default)]
pub(crate) struct DomainEvents {
    pub(crate) queue: VecDeque<DomainEvent>,
}

pub(crate) enum DomainEvent {
    Scene(SceneEvent),
    Camera(CameraEvent),
    Global(GlobalEvent),
    Assets(AssetEvent),
    Entity(EntityEvent),
    Selection(SelectionEvent),
}

pub(crate) enum SceneEvent {}

pub(crate) enum SelectionEvent {
    Selected(Option<Entity>),
}
pub(crate) enum AssetEvent {
    LoadGltf(PathBuf),
    ChangeSkybox(PathBuf),

}
pub(crate) enum EntityEvent {
    RemoveEntity(Entity),
    AddParent(Entity),
    UpdateTag(Entity, TagComponent),
    UpdateTransform(Entity, TransformComponent),
    UpdateMaterial(Entity, MaterialDesc),
    UpdateLight(Entity, LightComponent),
    
}
pub(crate) enum GlobalEvent {
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

pub(crate) enum CameraEvent {
    RecenterCamera,
    CameraFov(math::Rad<f32>),
    CameraDistance(f32),
    CameraNearFar((f32, f32)),
}
