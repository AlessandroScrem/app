use super::{
    EditorSettingsData, EditorStatisticsData, EntityData, EntityId, HierarchyData, IblData,
    InspectorData, SceneSettingsData, EditorResourceStatsData,
};

pub type QueryId = u64;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Query {
    Hierarchy,
    Entity { entity: EntityId },
    Children { parent: EntityId },
    Inspector { entity: EntityId },
    Selection,
    Settings,
    Statistics,
    ResourceStats,
    SceneSettings,
    Ibls,
}

#[derive(Clone, Debug)]
pub struct QueryRequest {
    pub id: QueryId,
    pub query: Query,
}

#[derive(Clone, Debug)]
pub struct QueryResponse {
    pub id: QueryId,
    pub result: QueryResult,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum QueryResult {
    Hierarchy(HierarchyData),
    Entity(Option<EntityData>),
    Children(Vec<EntityData>),
    Inspector(Option<InspectorData>),
    Selection(Vec<EntityId>),
    Settings(EditorSettingsData),
    Statistics(EditorStatisticsData),
    ResourceStats(EditorResourceStatsData),
    SceneSettings(SceneSettingsData),
    Ibls(Vec<IblData>),
}
