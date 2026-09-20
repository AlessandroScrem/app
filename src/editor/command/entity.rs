use crate::editor::{EditValue, EntityId};

#[derive(Clone, Debug)]
pub enum EntityCommand {
    AddLight,
    Remove { entities: Vec<EntityId> },
    AddParent { entity: EntityId },
    SetEnabled { entity: EntityId, enabled: bool },
    Edit { entity: EntityId, edit: EditValue },
    BeginTransformEdit { entity: EntityId },
    EndTransformEdit { entity: EntityId },
}
