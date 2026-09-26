use crate::editor::{
    EditorCommand, EditorEvent, EditorServiceChannels, EditorStatisticsData, EntityId, Query,
    QueryRequest, QueryResponse, QueryResult,
};
use crate::engine::engine::EventBus;

pub trait EditorBackend {
    fn query(&self, query: &Query) -> QueryResult;
    fn command(&mut self, command: EditorCommand, bus: &mut EventBus) -> Vec<EditorEvent>;
    fn editor_command(
        &mut self,
        command: crate::editor::EditorCommand,
        bus: &mut EventBus,
    ) -> Vec<EditorEvent>;
    fn entity_command(
        &mut self,
        command: crate::editor::EntityCommand,
        bus: &mut EventBus,
    ) -> Vec<EditorEvent>;
    fn selection_command(
        &mut self,
        command: crate::editor::SelectionCommand,
        bus: &mut EventBus,
    ) -> Vec<EditorEvent>;
    fn scene_command(
        &mut self,
        command: crate::editor::SceneCommand,
        bus: &mut EventBus,
    ) -> Vec<EditorEvent>;
    fn asset_command(
        &mut self,
        command: crate::editor::AssetCommand,
        bus: &mut EventBus,
    ) -> Vec<EditorEvent>;
    fn camera_command(
        &mut self,
        command: crate::editor::CameraCommand,
        bus: &mut EventBus,
    ) -> Vec<EditorEvent>;
    fn global_command(
        &mut self,
        command: crate::editor::GlobalCommand,
        bus: &mut EventBus,
    ) -> Vec<EditorEvent>;
    fn editor_scene_revision(&self) -> u64;
    fn editor_selection(&self) -> Vec<EntityId>;
    fn editor_entities(&self) -> Vec<EntityId>;
}
pub struct EditorService {
    channels: EditorServiceChannels,
    last_scene_revision: u64,
    last_selection: Vec<EntityId>,
    last_entities: Vec<EntityId>,
    statistics: EditorStatisticsData,
}
impl EditorService {
    pub fn new(channels: EditorServiceChannels) -> Self {
        Self {
            channels,
            last_scene_revision: 0,
            last_selection: Vec::new(),
            last_entities: Vec::new(),
            statistics: EditorStatisticsData::default(),
        }
    }
    pub fn set_statistics(&mut self, statistics: EditorStatisticsData) {
        if self.statistics != statistics {
            self.statistics = statistics;
            let _ = self.channels.event_tx.send(EditorEvent::StatisticsChanged);
        }
    }
    pub fn process<B: EditorBackend>(&mut self, backend: &mut B, bus: &mut EventBus) {
        while let Ok(command) = self.channels.command_rx.try_recv() {
            for event in backend.command(command, bus) {
                let _ = self.channels.event_tx.send(event);
            }
        }
        while let Ok(request) = self.channels.query_rx.try_recv() {
            self.respond(backend, request);
        }
        let entities = backend.editor_entities();
        for entity in entities
            .iter()
            .filter(|id| !self.last_entities.contains(id))
        {
            let _ = self
                .channels
                .event_tx
                .send(EditorEvent::EntityCreated { entity: *entity });
        }
        for entity in self
            .last_entities
            .iter()
            .filter(|id| !entities.contains(id))
        {
            let _ = self
                .channels
                .event_tx
                .send(EditorEvent::EntityDeleted { entity: *entity });
        }
        self.last_entities = entities;
        let scene_revision = backend.editor_scene_revision();
        if scene_revision != self.last_scene_revision {
            self.last_scene_revision = scene_revision;
            let _ = self.channels.event_tx.send(EditorEvent::SceneChanged);
        }
        let selection = backend.editor_selection();
        if selection != self.last_selection {
            self.last_selection = selection.clone();
            let _ = self.channels.event_tx.send(EditorEvent::SelectionChanged {
                entities: selection,
            });
        }
    }
    fn respond<B: EditorBackend>(&self, backend: &B, request: QueryRequest) {
        let result = match &request.query {
            Query::Statistics => QueryResult::Statistics(self.statistics.clone()),
            _ => backend.query(&request.query),
        };
        let _ = self.channels.response_tx.send(QueryResponse {
            id: request.id,
            result,
        });
    }
}
