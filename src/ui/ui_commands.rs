use crate::editor::{
    EditValue, EditorCommandClient, EditorConnection, EditorEdit, EditorEvent, EditorSettingsData,
    EditorStatisticsData, EntityId, HierarchyData, InspectorData, LightData, Query, QueryId,
    QueryResponse, QueryResult, SceneSettingsData, TransformData,
};
use std::collections::HashMap;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
enum QuerySlot {
    Hierarchy,
    Selection,
    Inspector,
    Settings,
    Statistics,
    SceneSettings,
}

pub(crate) struct UiCommands {
    connection: EditorConnection,
    hierarchy: Option<HierarchyData>,
    selection: Vec<EntityId>,
    inspector: Option<InspectorData>,
    settings: Option<EditorSettingsData>,
    statistics: Option<EditorStatisticsData>,
    scene_settings: SceneSettingsData,
    pending_queries: HashMap<QueryId, QuerySlot>,
    edit: Option<EditorEdit<EntityId, EditValue>>,
}

impl UiCommands {
    pub(crate) fn new(connection: EditorConnection) -> Self {
        Self {
            connection,
            hierarchy: None,
            selection: Vec::new(),
            inspector: None,
            settings: None,
            statistics: None,
            scene_settings: SceneSettingsData::default(),
            pending_queries: HashMap::new(),
            edit: None,
        }
    }

    pub(crate) fn command_client(&self) -> &EditorCommandClient {
        &self.connection.commands
    }

    pub(crate) fn hierarchy(&self) -> Option<&HierarchyData> {
        self.hierarchy.as_ref()
    }

    pub(crate) fn selection(&self) -> &[EntityId] {
        &self.selection
    }

    pub(crate) fn inspector(&self) -> Option<&InspectorData> {
        self.inspector.as_ref()
    }

    pub(crate) fn settings(&self) -> Option<&EditorSettingsData> {
        self.settings.as_ref()
    }

    pub(crate) fn statistics(&self) -> Option<&EditorStatisticsData> {
        self.statistics.as_ref()
    }

    pub(crate) fn scene_settings(&self) -> &SceneSettingsData {
        &self.scene_settings
    }

    pub(crate) fn take_edit(&mut self) -> Option<EditorEdit<EntityId, EditValue>> {
        self.edit.take()
    }

    pub(crate) fn set_edit(&mut self, edit: Option<EditorEdit<EntityId, EditValue>>) {
        self.edit = edit;
    }

    pub(crate) fn process(&mut self) {
        self.process_responses();
        self.process_events();
        self.ensure_queries();
    }

    fn is_editing_inspector(&self) -> bool {
        self.edit.is_some()
    }

    fn request(&mut self, slot: QuerySlot, query: Query) {
        self.remove_pending(slot);
        let id = self.connection.queries.request(query);
        self.pending_queries.insert(id, slot);
    }

    fn remove_pending(&mut self, slot: QuerySlot) {
        self.pending_queries.retain(|_, pending| *pending != slot);
    }

    fn request_initial_queries(&mut self) {
        self.request(QuerySlot::Hierarchy, Query::Hierarchy);
        self.request(QuerySlot::Selection, Query::Selection);
        self.request(QuerySlot::Settings, Query::Settings);
        self.request(QuerySlot::Statistics, Query::Statistics);
        self.request(QuerySlot::SceneSettings, Query::SceneSettings);
    }

    fn invalidate_all(&mut self) {
        self.hierarchy = None;
        self.settings = None;
        self.statistics = None;
        self.pending_queries.clear();

        if !self.is_editing_inspector() {
            self.inspector = None;
        }

        self.request_initial_queries();
        self.request_inspector();
    }

    fn request_inspector(&mut self) {
        if self.is_editing_inspector() {
            return;
        }

        if let [entity] = *self.selection.as_slice() {
            self.request(QuerySlot::Inspector, Query::Inspector { entity });
        } else {
            self.inspector = None;
            self.remove_pending(QuerySlot::Inspector);
        }
    }

    fn apply_query_response(&mut self, response: QueryResponse) {
        let Some(slot) = self.pending_queries.remove(&response.id) else {
            return;
        };

        match (slot, response.result) {
            (QuerySlot::Hierarchy, QueryResult::Hierarchy(data)) => self.hierarchy = Some(data),
            (QuerySlot::Settings, QueryResult::Settings(data)) => self.settings = Some(data),
            (QuerySlot::Statistics, QueryResult::Statistics(data)) => self.statistics = Some(data),
            (QuerySlot::SceneSettings, QueryResult::SceneSettings(data)) => {
                self.scene_settings = data;
            }
            (QuerySlot::Selection, QueryResult::Selection(selection)) => {
                self.selection = selection;
                self.request_inspector();
            }
            (QuerySlot::Inspector, QueryResult::Inspector(data)) => {
                if !self.is_editing_inspector() {
                    self.inspector = data;
                }
            }
            _ => {}
        }
    }

    fn apply_transform_changed(&mut self, entity: EntityId, transform: TransformData) {
        if let Some(inspector) = &mut self.inspector {
            if inspector.entity == entity {
                inspector.transform = transform.clone();
            }
        }

        if let Some(edit) = &mut self.edit {
            if edit.key == entity {
                if let EditValue::Transform(current) = &mut edit.value {
                    *current = transform;
                }
            }
        } else {
            self.request(QuerySlot::Inspector, Query::Inspector { entity });
        }
    }

    fn apply_name_changed(&mut self, entity: EntityId, name: String) {
        self.request(QuerySlot::Hierarchy, Query::Hierarchy);

        if let Some(inspector) = &mut self.inspector {
            if inspector.entity == entity {
                inspector.name = name.clone();
            }
        }

        if let Some(edit) = &mut self.edit {
            if edit.key == entity {
                if let EditValue::Name(current) = &mut edit.value {
                    *current = name;
                }
            }
        }
    }

    fn apply_light_changed(&mut self, entity: EntityId, light: LightData) {
        if let Some(inspector) = &mut self.inspector {
            if inspector.entity == entity {
                inspector.light = Some(light.clone());
            }
        }

        if let Some(edit) = &mut self.edit {
            if edit.key == entity {
                if let EditValue::Light(current) = &mut edit.value {
                    *current = light;
                }
            }
        }
    }

    fn apply_event(&mut self, event: EditorEvent) {
        match event {
            EditorEvent::SceneChanged
            | EditorEvent::EntityCreated { .. }
            | EditorEvent::EntityDeleted { .. } => self.invalidate_all(),
            EditorEvent::SelectionChanged { entities } => {
                self.selection = entities;
                self.request_inspector();
            }
            EditorEvent::TransformChanged { entity, transform } => {
                self.apply_transform_changed(entity, transform);
            }
            EditorEvent::NameChanged { entity, name } => {
                self.apply_name_changed(entity, name);
            }
            EditorEvent::LightChanged { entity, light } => {
                self.apply_light_changed(entity, light);
            }
            EditorEvent::SettingsChanged => {
                self.settings = None;
                self.request(QuerySlot::Settings, Query::Settings);
            }
            EditorEvent::StatisticsChanged => {
                self.statistics = None;
                self.request(QuerySlot::Statistics, Query::Statistics);
            }
        }
    }

    fn process_responses(&mut self) {
        while let Some(response) = self.connection.try_recv_response() {
            self.apply_query_response(response);
        }
    }

    fn process_events(&mut self) {
        while let Some(event) = self.connection.events.try_recv() {
            self.apply_event(event);
        }
    }

    fn ensure_queries(&mut self) {
        self.ensure_hierarchy();
        self.ensure_settings();
        self.ensure_statistics();
    }

    fn ensure_hierarchy(&mut self) {
        if self.hierarchy.is_none() && !self.has_pending(QuerySlot::Hierarchy) {
            self.request(QuerySlot::Hierarchy, Query::Hierarchy);
        }
    }

    fn ensure_settings(&mut self) {
        if self.settings.is_none() && !self.has_pending(QuerySlot::Settings) {
            self.request(QuerySlot::Settings, Query::Settings);
        }
    }

    fn ensure_statistics(&mut self) {
        if self.statistics.is_none() && !self.has_pending(QuerySlot::Statistics) {
            self.request(QuerySlot::Statistics, Query::Statistics);
        }
    }

    fn has_pending(&self, slot: QuerySlot) -> bool {
        self.pending_queries.values().any(|pending| *pending == slot)
    }
}
