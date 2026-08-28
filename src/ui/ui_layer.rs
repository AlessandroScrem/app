use super::*;
use crate::editor::{
    EditorConnection, EditorEdit, EditorEvent, EditorSettingsData, EditorStatisticsData, EntityId,
    HierarchyData, InspectorData, LightData, Query, QueryId, QueryResult, TransformData,
};
use imgui::*;
use imgui_winit_support::WinitPlatform;
use std::collections::HashMap;
use winit::event::Event;
use winit::window::Window;

#[derive(Clone, Copy, Debug)]
pub enum EditorInteraction {
    None,
    Selecting {
        start: crate::math::Vec2,
        current: crate::math::Vec2,
    },
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
enum QuerySlot {
    Hierarchy,
    Selection,
    Inspector,
    Settings,
    Statistics,
}

pub struct UiContext<'a> {
    pub connection: &'a mut EditorConnection,
    pub hierarchy: Option<&'a HierarchyData>,
    pub selection: &'a [EntityId],
    pub inspector: Option<&'a InspectorData>,
    pub settings: Option<&'a EditorSettingsData>,
    pub statistics: Option<&'a EditorStatisticsData>,
    pub transform_edit: &'a mut Option<EditorEdit<EntityId, TransformData>>,
    pub light_edit: &'a mut Option<EditorEdit<EntityId, LightData>>,
    pub name_edit: &'a mut Option<EditorEdit<EntityId, String>>,
    pub editor_interaction: &'a EditorInteraction,
}

pub struct UiLayer {
    context: imgui::Context,
    pub platform: WinitPlatform,
    ini_loaded: bool,
    timestep: crate::timestep::Timestep,
    stack: UiStack,
    #[allow(dead_code)]
    adapter_string: String,
    pub connection: EditorConnection,
    hierarchy: Option<HierarchyData>,
    selection: Vec<EntityId>,
    inspector: Option<InspectorData>,
    settings: Option<EditorSettingsData>,
    statistics: Option<EditorStatisticsData>,
    latest_queries: HashMap<QuerySlot, QueryId>,
    transform_edit: Option<EditorEdit<EntityId, TransformData>>,
    light_edit: Option<EditorEdit<EntityId, LightData>>,
    name_edit: Option<EditorEdit<EntityId, String>>,
    editor_interaction: EditorInteraction,
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

struct ViewportUi;
impl Layer for ViewportUi {
    fn build(&mut self, ui: &Ui, ctx: &mut UiContext) {
        let EditorInteraction::Selecting { start, current } = *ctx.editor_interaction else {
            return;
        };
        let scale = ui.io().display_framebuffer_scale;
        let start = [start.x / scale[0], start.y / scale[1]];
        let current = [current.x / scale[0], current.y / scale[1]];
        ui.get_foreground_draw_list()
            .add_rect(start, current, [1.0, 0.0, 0.0, 1.0])
            .thickness(1.0)
            .build();
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
        ui.push(ViewportUi);
        ui.push(crate::ui::menu_bar::MenuBarUi);
        ui.push(EntityListUi);
        ui.push(PropertyUi);
        ui.push(crate::ui::settings::SettingsUi::default());
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
            latest_queries: HashMap::new(),
            transform_edit: None,
            light_edit: None,
            name_edit: None,
            editor_interaction: EditorInteraction::None,
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
    pub fn set_editor_interaction(&mut self, interaction: EditorInteraction) {
        self.editor_interaction = interaction;
    }

    fn is_editing_inspector(&self) -> bool {
        self.transform_edit.is_some() || self.light_edit.is_some() || self.name_edit.is_some()
    }

    fn request(&mut self, slot: QuerySlot, query: Query) {
        let id = self.connection.queries.request(query);
        self.latest_queries.insert(slot, id);
    }
    fn invalidate_all(&mut self) {
        self.hierarchy = None;
        self.inspector = None;
        self.settings = None;
        self.statistics = None;
        self.latest_queries.clear();
        self.request(QuerySlot::Hierarchy, Query::Hierarchy);
        self.request(QuerySlot::Selection, Query::Selection);
        self.request(QuerySlot::Settings, Query::Settings);
        self.request(QuerySlot::Statistics, Query::Statistics);
        if let [entity] = *self.selection.as_slice() {
            println!("Selection len: {}", self.selection.len());
            self.request(QuerySlot::Inspector, Query::Inspector { entity });
        }
    }
    fn process_connection(&mut self) {
        while let Some(response) = self.connection.try_recv_response() {
            let Some(slot) = self
                .latest_queries
                .iter()
                .find_map(|(slot, id)| (*id == response.id).then_some(*slot))
            else {
                continue;
            };
            match (slot, response.result) {
                (QuerySlot::Hierarchy, QueryResult::Hierarchy(data)) => self.hierarchy = Some(data),
                (QuerySlot::Selection, QueryResult::Selection(selection)) => {
                    self.selection = selection;
                    if !self.is_editing_inspector() {
                        if let [entity] = *self.selection.as_slice() {
                            self.request(QuerySlot::Inspector, Query::Inspector { entity });
                        }
                    }
                }
                (QuerySlot::Inspector, QueryResult::Inspector(data)) => {
                    if !self.is_editing_inspector() {
                        self.inspector = data;
                    }
                }
                (QuerySlot::Settings, QueryResult::Settings(data)) => self.settings = Some(data),
                (QuerySlot::Statistics, QueryResult::Statistics(data)) => {
                    self.statistics = Some(data)
                }
                _ => {}
            }
        }
        while let Some(event) = self.connection.events.try_recv() {
            match event {
                EditorEvent::SceneChanged
                | EditorEvent::EntityCreated { .. }
                | EditorEvent::EntityDeleted { .. } => self.invalidate_all(),
                EditorEvent::SelectionChanged { entities } => {
                    self.selection = entities;
                    if !self.is_editing_inspector() {
                        if let [entity] = *self.selection.as_slice() {
                            println!("Selection len: {}", self.selection.len());
                            self.request(QuerySlot::Inspector, Query::Inspector { entity });
                        }
                    }
                }
                EditorEvent::TransformChanged { entity, transform } => {
                    if let Some(inspector) = &mut self.inspector {
                        if inspector.entity == entity {
                            inspector.transform = transform.clone();
                        }
                    }
                    if let Some(edit) = &mut self.transform_edit {
                        if edit.key == entity {
                            edit.value = transform;
                        }
                    } else {
                        self.request(QuerySlot::Inspector, Query::Inspector { entity });
                    }
                }
                EditorEvent::NameChanged { entity, .. } | EditorEvent::LightChanged { entity } => {
                    if !self.is_editing_inspector() && [entity] == *self.selection.as_slice() {
                        self.latest_queries.remove(&QuerySlot::Inspector);
                        self.request(QuerySlot::Inspector, Query::Inspector { entity });
                    }

                    self.latest_queries.remove(&QuerySlot::Hierarchy);
                    self.request(QuerySlot::Hierarchy, Query::Hierarchy);
                }
                EditorEvent::SettingsChanged => {
                    self.latest_queries.remove(&QuerySlot::Settings);
                    self.request(QuerySlot::Settings, Query::Settings);
                }
                EditorEvent::StatisticsChanged => {
                    self.latest_queries.remove(&QuerySlot::Statistics);
                    self.request(QuerySlot::Statistics, Query::Statistics);
                }
            }
        }
        if self.hierarchy.is_none() && !self.latest_queries.contains_key(&QuerySlot::Hierarchy) {
            self.request(QuerySlot::Hierarchy, Query::Hierarchy);
        }
        if !self.latest_queries.contains_key(&QuerySlot::Selection) {
            self.request(QuerySlot::Selection, Query::Selection);
        }
        if self.settings.is_none() && !self.latest_queries.contains_key(&QuerySlot::Settings) {
            self.request(QuerySlot::Settings, Query::Settings);
        }
        if self.statistics.is_none() && !self.latest_queries.contains_key(&QuerySlot::Statistics) {
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
        let selection = &self.selection;
        let editor_interaction = &self.editor_interaction;
        let mut transform_edit = self.transform_edit.take();
        let mut light_edit = self.light_edit.take();
        let mut name_edit = self.name_edit.take();
        let mut ctx = UiContext {
            connection: &mut self.connection,
            hierarchy,
            selection,
            inspector,
            settings,
            statistics,
            transform_edit: &mut transform_edit,
            light_edit: &mut light_edit,
            name_edit: &mut name_edit,
            editor_interaction,
        };
        self.stack.build(ui, &mut ctx);
        self.transform_edit = transform_edit;
        self.light_edit = light_edit;
        self.name_edit = name_edit;
        self.platform.prepare_render(ui, window);
        self.end_frame();
    }
}
