use super::types::*;

#[derive(Clone, Debug)]
pub enum EditorEvent {
    SelectionChanged(Vec<EntityRef>),
    EntityChanged(EntityRef),
}

pub trait EventClient {
    fn poll_events(&mut self) -> Vec<EditorEvent>;
}
