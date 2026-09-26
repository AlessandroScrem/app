use crate::editor::{EditValue, EntityId, TransformData};

#[derive(Clone, Debug)]
pub enum EntityCommand {
    AddLight,
    Remove { entities: Vec<EntityId> },
    AddParent { entity: EntityId },
    SetEnabled { entity: EntityId, enabled: bool },
    Edit { entity: EntityId, edit: EditValue },
    SetTransform { entity: EntityId, transform: TransformData },
    BeginTransformEdit { entity: EntityId },
    EndTransformEdit { entity: EntityId },
}
