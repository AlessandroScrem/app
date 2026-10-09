use crate::editor::{EntityId, LightData, TransformData};

#[derive(Clone, Debug)]
pub enum EntityCommand {
    AddLight,
    Remove {
        entities: Vec<EntityId>,
    },
    AddParent {
        entity: EntityId,
    },
    SetEnabled {
        entity: EntityId,
        enabled: bool,
    },
    SetName {
        entity: EntityId,
        name: String,
    },
    SetLight {
        entity: EntityId,
        light: LightData,
    },
    SetTransform {
        entity: EntityId,
        transform: TransformData,
    },
    BeginTransformEdit {
        entity: EntityId,
    },
    EndTransformEdit {
        entity: EntityId,
    },
}
