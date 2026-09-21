use crate::editor::EntityId;

#[derive(Clone, Debug)]
pub enum SelectionCommand {
    Set { entities: Vec<EntityId> },
    // Clear,
    // Add { entity: EntityId },
    // Remove { entity: EntityId },
    // Toggle { entity: EntityId },
    // SelectHovered,
    // Drag {
    //     position: (u32, u32),
    //     size: (u32, u32),
    // },
}
