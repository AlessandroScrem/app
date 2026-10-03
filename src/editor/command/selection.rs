use crate::editor::EntityId;

#[derive(Clone, Debug)]
pub enum SelectionCommand {
    Select { entities: Vec<EntityId> },
    SelectIbl { id: crate::assets::IblId },
}
