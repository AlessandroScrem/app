mod asset;
mod camera;
mod entity;
mod global;
mod scene;
mod selection;

pub use asset::AssetCommand;
pub use camera::CameraCommand;
pub use entity::EntityCommand;
pub use global::GlobalCommand;
pub use scene::SceneCommand;
pub use selection::SelectionCommand;

use crate::editor::{EditValue, EntityId, LightData, TransformData};
use std::path::PathBuf;

/// Top-level command API kept stable while command implementations are split by domain.
#[derive(Clone, Debug)]
pub enum EditorCommand {
    Select {
        entities: Vec<EntityId>,
    },
    SetTransform {
        entity: EntityId,
        transform: TransformData,
    },
    SetName {
        entity: EntityId,
        name: String,
    },
    SetLight {
        entity: EntityId,
        light: LightData,
    },
    Delete {
        entities: Vec<EntityId>,
    },
    BeginTransformEdit {
        entity: EntityId,
    },
    EndTransformEdit {
        entity: EntityId,
    },
    AddLight,
    AddParent {
        entity: EntityId,
    },
    SetEntityEnabled {
        entity: EntityId,
        enabled: bool,
    },
    LoadGltf {
        path: PathBuf,
    },
    OpenScene {
        path: PathBuf,
    },
    SaveScene,
    SaveSceneAs {
        path: PathBuf,
    },
    ClearScene,
    Exit,
    SetLightEnable(bool),
    SetIblEnable(bool),
    SetSkyboxEnable(bool),
    SetSkyboxBlur(bool),
    SetAxisEnable(bool),
    SetBoundingBoxEnable(bool),
    SetBoundingBoxAxisAligned(bool),
    SetMipsWithCompute(bool),
    SetEnvironmentRotation(f32),
    SetDebugCode(u32),
    SetExposure(f32),
    SetIblIntensity(f32),
    SetTonemap(u32),
    RecenterCamera,
    SetCameraFov(f32),
    SetCameraDistance(f32),
    SetCameraNearFar {
        near: f32,
        far: f32,
    },
    AddIbl {
        path: PathBuf,
    },
    DragSelection((u32, u32), (u32, u32)),
}

impl EditorCommand {
    /// Returns the domain-specific representation when a command has been migrated.
    pub fn domain(&self) -> Option<DomainCommand> {
        match self {
            Self::Select { entities } => Some(DomainCommand::Selection(SelectionCommand::Set {
                entities: entities.clone(),
            })),
            Self::SetTransform { entity, transform } => {
                Some(DomainCommand::Entity(EntityCommand::Edit {
                    entity: *entity,
                    edit: EditValue::Transform(transform.clone()),
                }))
            }
            Self::SetName { entity, name } => Some(DomainCommand::Entity(EntityCommand::Edit {
                entity: *entity,
                edit: EditValue::Name(name.clone()),
            })),
            Self::SetLight { entity, light } => Some(DomainCommand::Entity(EntityCommand::Edit {
                entity: *entity,
                edit: EditValue::Light(light.clone()),
            })),
            Self::Delete { entities } => Some(DomainCommand::Entity(EntityCommand::Remove {
                entities: entities.clone(),
            })),
            Self::BeginTransformEdit { entity } => {
                Some(DomainCommand::Entity(EntityCommand::BeginTransformEdit {
                    entity: *entity,
                }))
            }
            Self::EndTransformEdit { entity } => {
                Some(DomainCommand::Entity(EntityCommand::EndTransformEdit {
                    entity: *entity,
                }))
            }
            Self::AddLight => Some(DomainCommand::Entity(EntityCommand::AddLight)),
            Self::AddParent { entity } => Some(DomainCommand::Entity(EntityCommand::AddParent {
                entity: *entity,
            })),
            Self::SetEntityEnabled { entity, enabled } => {
                Some(DomainCommand::Entity(EntityCommand::SetEnabled {
                    entity: *entity,
                    enabled: *enabled,
                }))
            }
            Self::LoadGltf { path } => {
                Some(DomainCommand::Asset(AssetCommand::LoadGltf(path.clone())))
            }
            Self::OpenScene { path } => {
                Some(DomainCommand::Scene(SceneCommand::Open(path.clone())))
            }
            Self::SaveScene => Some(DomainCommand::Scene(SceneCommand::Save)),
            Self::SaveSceneAs { path } => {
                Some(DomainCommand::Scene(SceneCommand::SaveAs(path.clone())))
            }
            Self::ClearScene => Some(DomainCommand::Scene(SceneCommand::Clear)),
            Self::SetLightEnable(value) => {
                Some(DomainCommand::Global(GlobalCommand::SetLightEnable(*value)))
            }
            Self::SetIblEnable(value) => {
                Some(DomainCommand::Global(GlobalCommand::SetIblEnable(*value)))
            }
            Self::SetSkyboxEnable(value) => Some(DomainCommand::Global(
                GlobalCommand::SetSkyboxEnable(*value),
            )),
            Self::SetSkyboxBlur(value) => {
                Some(DomainCommand::Global(GlobalCommand::SetSkyboxBlur(*value)))
            }
            Self::SetAxisEnable(value) => {
                Some(DomainCommand::Global(GlobalCommand::SetAxisEnable(*value)))
            }
            Self::SetBoundingBoxEnable(value) => Some(DomainCommand::Global(
                GlobalCommand::SetBoundingBoxEnable(*value),
            )),
            Self::SetBoundingBoxAxisAligned(value) => Some(DomainCommand::Global(
                GlobalCommand::SetBoundingBoxAxisAligned(*value),
            )),
            Self::SetMipsWithCompute(value) => Some(DomainCommand::Global(
                GlobalCommand::SetMipsWithCompute(*value),
            )),
            Self::SetEnvironmentRotation(value) => Some(DomainCommand::Global(
                GlobalCommand::SetEnvironmentRotation(*value),
            )),
            Self::SetDebugCode(value) => {
                Some(DomainCommand::Global(GlobalCommand::SetDebugCode(*value)))
            }
            Self::SetExposure(value) => {
                Some(DomainCommand::Global(GlobalCommand::SetExposure(*value)))
            }
            Self::SetIblIntensity(value) => Some(DomainCommand::Global(
                GlobalCommand::SetIblIntensity(*value),
            )),
            Self::SetTonemap(value) => {
                Some(DomainCommand::Global(GlobalCommand::SetTonemap(*value)))
            }
            Self::RecenterCamera => Some(DomainCommand::Camera(CameraCommand::Recenter)),
            Self::SetCameraFov(value) => Some(DomainCommand::Camera(CameraCommand::SetFov(*value))),
            Self::SetCameraDistance(value) => {
                Some(DomainCommand::Camera(CameraCommand::SetDistance(*value)))
            }
            Self::SetCameraNearFar { near, far } => {
                Some(DomainCommand::Camera(CameraCommand::SetNearFar {
                    near: *near,
                    far: *far,
                }))
            }
            Self::AddIbl { path } => Some(DomainCommand::Asset(AssetCommand::AddIbl(path.clone()))),
            Self::Exit | Self::DragSelection(_, _) => None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum DomainCommand {
    Scene(SceneCommand),
    Selection(SelectionCommand),
    Entity(EntityCommand),
    Asset(AssetCommand),
    Camera(CameraCommand),
    Global(GlobalCommand),
}
