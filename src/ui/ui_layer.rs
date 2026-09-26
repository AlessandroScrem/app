use super::*;
use crate::editor::{
    EditValue, EditorConnection, EditorEdit, EditorEvent, EditorSettingsData, EditorStatisticsData,
    EntityId, HierarchyData, InspectorData, LightData, Query, QueryId, QueryResponse, QueryResult,
    SceneSettingsData, TransformData,
};

use imgui::Ui;
use imgui_winit_support::WinitPlatform;
use winit::event::Event;
use winit::window::Window;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
enum QuerySlot {
    Hierarchy,
    Selection,
    Inspector,
    Settings,
    Statistics,
    SceneSettings,
}

pub struct UiContext<'a> {
    pub connection: &'a mut EditorConnection,
    pub hierarchy: Option<&'a HierarchyData>,
    pub selection: &'a [EntityId],
    pub inspector: Option<&'a InspectorData>,
    pub settings: Option<&'a EditorSettingsData>,
    pub statistics: Option<&'a EditorStatisticsData>,
    pub edit: &'a mut Option<EditorEdit<EntityId, EditValue>>,
    pub scene_settings: &'a SceneSettingsData,
    pub adapter_string: &'a String,
}

pub struct UiLayer {
    context: imgui::Context,
    pub platform: WinitPlatform,
    ini_loaded: bool,
    timestep: crate::timestep::Timestep,
    stack: UiStack,
    adapter_string: String,
    pub connection: EditorConnection,
    hierarchy: Option<HierarchyData>,
    selection: Vec<EntityId>,
    inspector: Option<InspectorData>,
    settings: Option<EditorSettingsData>,
    statistics: Option<EditorStatisticsData>,
    scene_settings: SceneSettingsData,
    pending_queries: HashMap<QueryId, QuerySlot>,
    edit: Option<EditorEdit<EntityId, EditValue>>,
}

struct UiStack {
    layers: Vec<Box<dyn Layer>>,
}

impl UiStack {
    fn new() -> Self {
        Self { layers: Vec::new() }
    }
    fn push<L: Layer + 'static>(&mut self, layer: L) {
        self.layers.push(Box::new(layer));
    }
}

pub trait Layer {
    fn build(&mut self, ui: &Ui, ctx: &mut UiContext);
}

impl Layer for UiStack {
    fn build(&mut self, ui: &Ui, ctx: &mut UiContext) {
        for layer in self.layers.iter_mut() {
            layer.build(ui, ctx);
        }
    }
}

impl UiLayer {
    pub fn new(
        window: &Window,
        mut context: imgui::Context,
        adapter_string: String,
        connection: EditorConnection,
    ) -> Self {
        tools::set_dark_theme_colors(context.style_mut());
        let io = context.io_mut();
        io.config_flags.insert(imgui::ConfigFlags::DOCKING_ENABLE);
        io.config_flags.insert(imgui::ConfigFlags::VIEWPORTS_ENABLE);
        context.set_ini_filename(None);
        let mut platform = WinitPlatform::new(&mut context);
        platform.attach_window(
            context.io_mut(),
            window,
            imgui_winit_support::HiDpiMode::Default,
        );
        let mut ui = UiStack::new();
        ui.push(ViewportUi::default());
        ui.push(MenuBarUi);
        ui.push(EntityListUi);
        ui.push(PropertyUi);
        ui.push(SettingsUi::default());
        Self {
            context,
            platform,
            ini_loaded: false,
            timestep: crate::timestep::Timestep::new(),
            stack: ui,
            adapter_string,
            connection,
            hierarchy: None,
            selection: Vec::new(),
            inspector: None,
            settings: None,
            statistics: None,
            pending_queries: HashMap::new(),
            edit: None,
            scene_settings: SceneSettingsData::default(),
        }
    }

    pub fn want_capture_mouse(&self) -> bool {
        self.context.io().want_capture_mouse
    }

    pub fn handle_event<T>(&mut self, window: &Window, event: &Event<T>) {
        self.platform
            .handle_event::<T>(self.context.io_mut(), window, event);
    }

    pub fn get_draw_data(&mut self) -> &imgui::DrawData {
        self.context.render()
    }

    fn is_editing_inspector(&self) -> bool {
        self.edit.is_some()
    }

    fn request(&mut self, slot: QuerySlot, query: Query) {
        let id = self.connection.queries.request(query);
        self.pending_queries.insert(id, slot);
    }

    fn request_initial_queries(&mut self) {
        self.request(QuerySlot::Hierarchy, Query::Hierarchy);
        self.request(QuerySlot::Selection, Query::Selection);
        self.request(QuerySlot::Settings, Query::Settings);
        self.request(QuerySlot::Statistics, Query::Statistics);
        self.request(QuerySlot::SceneSettings, Query::SceneSettings);
    }

    fn remove_pending_queries(&mut self) {
        self.pending_queries.clear();
    }

    fn invalidate_all(&mut self) {
        self.hierarchy = None;
        self.settings = None;
        self.statistics = None;
        
        self.remove_pending_queries();

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

    fn update_inspector(&mut self) {
        self.request_inspector();
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
                self.update_inspector();
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
        self.ensure_selection();
        self.ensure_settings();
        self.ensure_statistics();
    }

    fn process_connection(&mut self) {
        self.process_responses();
        self.process_events();
        self.ensure_queries();
    }

    fn has_pending(&self, slot: QuerySlot) -> bool {
        self.pending_queries
            .values()
            .any(|pending| *pending == slot)
    }

    fn ensure_hierarchy(&mut self) {
        if self.hierarchy.is_none() && !self.has_pending(QuerySlot::Hierarchy) {
            self.request(QuerySlot::Hierarchy, Query::Hierarchy);
        }
    }

    fn ensure_selection(&mut self) {
        if !self.has_pending(QuerySlot::Selection) {
            self.request(QuerySlot::Selection, Query::Selection);
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

    fn begin_frame(&mut self, window: &Window) {
        self.timestep.update();
        self.context
            .io_mut()
            .update_delta_time(self.timestep.delta());
        self.platform
            .prepare_frame(self.context.io_mut(), window)
            .expect("failed to prepare frame");
    }

    fn end_frame(&mut self) {
        if !self.ini_loaded {
            self.context.set_ini_filename(Some("imgui.ini".into()));
            if let Ok(content) = std::fs::read_to_string("imgui.ini") {
                self.context.load_ini_settings(&content);
            }
            self.ini_loaded = true;
        }
    }

    pub fn build(&mut self, window: &Window) {
        self.process_connection();
        self.begin_frame(window);
        let ui = self.context.frame();
        ui.dockspace_over_main_viewport();
        let hierarchy = self.hierarchy.as_ref();
        let inspector = self.inspector.as_ref();
        let settings = self.settings.as_ref();
        let statistics = self.statistics.as_ref();
        let scene_settings = &self.scene_settings;
        let selection = &self.selection;
        let mut edit = self.edit.take();
        let mut ctx = UiContext {
            connection: &mut self.connection,
            hierarchy,
            selection,
            inspector,
            settings,
            statistics,
            edit: &mut edit,
            scene_settings,
            adapter_string: &self.adapter_string,
        };
        self.stack.build(ui, &mut ctx);
        self.edit = edit;
        self.platform.prepare_render(ui, window);
        self.end_frame();
    }
}
