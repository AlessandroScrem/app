use super::types::*;

#[derive(Clone, Debug)]
pub enum EditorQuery {
    RootEntities,
    Entity(EntityRef),
    SelectedEntities,
}

#[derive(Clone, Debug)]
pub enum QueryResult {
    Entities(Vec<EntityState>),
    Entity(Option<EntityState>),
}

pub trait QueryClient {
    fn query(&mut self, query: EditorQuery) -> QueryResult;
}
