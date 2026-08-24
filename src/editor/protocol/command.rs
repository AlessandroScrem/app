use super::types::*;

#[derive(Clone, Debug)]
pub enum EditorCommand {
    Select(Vec<EntityRef>),
    SetTransform {
        entity: EntityRef,
        transform: TransformState,
    },
}

pub trait CommandClient {
    fn execute(&mut self, command: EditorCommand);
}
