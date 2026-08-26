use crate::app::domain::events::{DomainEvent, EntityEvent, SelectionEvent};
use crate::editor::{EditorCommand, EditorEvent, EditorServiceChannels, EditorStatisticsData, Query, QueryRequest, QueryResponse, QueryResult, EntityId};
use crate::engine::engine::EventBus;

pub trait EditorBackend { fn query(&self, query: &Query) -> QueryResult; fn command(&mut self, command: EditorCommand, bus: &mut EventBus) -> Vec<EditorEvent>; fn editor_scene_revision(&self) -> u64; fn editor_selection(&self) -> Vec<EntityId>; fn editor_entities(&self) -> Vec<EntityId>; }
pub struct EditorService { channels: EditorServiceChannels, last_scene_revision: u64, last_selection: Vec<EntityId>, last_entities: Vec<EntityId>, statistics: EditorStatisticsData }
impl EditorService {
    pub fn new(channels: EditorServiceChannels) -> Self { Self { channels, last_scene_revision: 0, last_selection: Vec::new(), last_entities: Vec::new(), statistics: EditorStatisticsData::default() } }
    pub fn set_statistics(&mut self, statistics: EditorStatisticsData) { if self.statistics != statistics { self.statistics = statistics; let _ = self.channels.event_tx.send(EditorEvent::StatisticsChanged); } }
    pub fn process<B: EditorBackend>(&mut self, backend: &mut B, bus: &mut EventBus) {
        while let Ok(command) = self.channels.command_rx.try_recv() { for event in backend.command(command, bus) { let _ = self.channels.event_tx.send(event); } }
        while let Ok(request) = self.channels.query_rx.try_recv() { self.respond(backend, request); }
        let entities = backend.editor_entities(); for entity in entities.iter().filter(|id| !self.last_entities.contains(id)) { let _ = self.channels.event_tx.send(EditorEvent::EntityCreated { entity: *entity }); } for entity in self.last_entities.iter().filter(|id| !entities.contains(id)) { let _ = self.channels.event_tx.send(EditorEvent::EntityDeleted { entity: *entity }); } self.last_entities = entities;
        let scene_revision = backend.editor_scene_revision(); if scene_revision != self.last_scene_revision { self.last_scene_revision = scene_revision; let _ = self.channels.event_tx.send(EditorEvent::SceneChanged); }
        let selection = backend.editor_selection(); if selection != self.last_selection { self.last_selection = selection.clone(); let _ = self.channels.event_tx.send(EditorEvent::SelectionChanged { entities: selection }); }
    }
    fn respond<B: EditorBackend>(&self, backend: &B, request: QueryRequest) { let result = match &request.query { Query::Statistics => QueryResult::Statistics(self.statistics.clone()), _ => backend.query(&request.query) }; let _ = self.channels.response_tx.send(QueryResponse { id: request.id, result }); }
}
pub(crate) fn select_command(entities: &[EntityId], bus: &mut EventBus) { let entity = entities.first().copied().map(crate::EntityRawU64::from_raw_u64); bus.send_domain(DomainEvent::Selection(SelectionEvent::Select(entity))); bus.send_domain(DomainEvent::Selection(SelectionEvent::SelectMulti(entities.to_vec()))); }
pub(crate) fn set_transform_command(entity: EntityId, transform: crate::editor::TransformData, bus: &mut EventBus) { let entity = crate::EntityRawU64::from_raw_u64(entity); let transform = crate::ecs::components::TransformComponent { position: transform.translation, rotation: transform.rotation, scale: transform.scale }; bus.send_domain(DomainEvent::Entity(EntityEvent::UpdateTransform(entity, transform))); }
