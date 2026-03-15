use std::{collections::VecDeque, path::PathBuf};
use crate::{assets::MaterialId, prelude::*};

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
    UpdateMaterial(MaterialId, MaterialDesc),
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
