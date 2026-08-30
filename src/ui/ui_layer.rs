use super::*;
use crate::editor::{
    EditValue, EditorCommand, EditorConnection, EditorEdit, EditorEvent, EditorSettingsData,
    EditorStatisticsData, EntityId, SceneSettingsData, HierarchyData, InspectorData, Query, QueryId, QueryResult,
};

use imgui::*;
use imgui_winit_support::WinitPlatform;
use std::collections::HashMap;
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
    scene_settings: SceneSettingsData,
    latest_queries: HashMap<QuerySlot, QueryId>,
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

#[derive(Debug, Default, Clone)]
struct ViewportUi {
    click_pos: Option<[f32; 2]>,
}

impl Layer for ViewportUi {
    fn build(&mut self, ui: &Ui, ctx: &mut UiContext) {
        if ui.io().want_capture_mouse {
            return;
        }

        match self.click_pos {
            None => {
                if ui.is_mouse_clicked(MouseButton::Left) && ui.is_key_down(Key::LeftCtrl) {
                    self.click_pos = Some(ui.io().mouse_pos);
                }
            }
            Some(start) => {
                let current = ui.io().mouse_pos;
                if ui.is_mouse_dragging(MouseButton::Left) && ui.is_key_down(Key::LeftCtrl) {
                    ui.get_foreground_draw_list()
                        .add_rect(start, current, [1.0, 0.0, 0.0, 1.0])
                        .thickness(1.0)
                        .build();
                }

                if ui.is_mouse_released(MouseButton::Left) {
                    let scale = ui.io().display_framebuffer_scale;
                    let start = [start[0] * scale[0], start[1] * scale[1]];
                    let current = [current[0] * scale[0], current[1] * scale[1]];

                    let pos = (
                        start[0].min(current[0]) as u32,
                        start[1].min(current[1]) as u32,
                    );
                    let width = (start[0] - current[0]).abs() as u32;
                    let height = (start[1] - current[1]).abs() as u32;
                    let size = (width, height);

                    ctx.connection
                        .commands
                        .send(EditorCommand::DragSelection(pos, size));

                    self.click_pos = None;
                }
            }
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
        self.latest_queries.insert(slot, id);
    }
    fn invalidate_all(&mut self) {
        if !self.is_editing_inspector() {
            self.inspector = None;
        }

        self.hierarchy = None;
        self.settings = None;
        self.statistics = None;

        self.latest_queries.clear();

        self.request(QuerySlot::Hierarchy, Query::Hierarchy);
        self.request(QuerySlot::Selection, Query::Selection);
        self.request(QuerySlot::Settings, Query::Settings);
        self.request(QuerySlot::Statistics, Query::Statistics);
        self.request(QuerySlot::SceneSettings, Query::SceneSettings);

        if !self.is_editing_inspector() {
            if let [entity] = *self.selection.as_slice() {
                self.request(QuerySlot::Inspector, Query::Inspector { entity });
            }
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
                (QuerySlot::SceneSettings, QueryResult::SceneSettings(data)) => {
                    self.scene_settings = data;
                    println!("Scene settings updated: {:?}", self.scene_settings);
                },
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
                            self.request(QuerySlot::Inspector, Query::Inspector { entity });
                        } else {
                            self.inspector = None;
                        }
                    }
                }
                EditorEvent::TransformChanged { entity, transform } => {
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
                EditorEvent::NameChanged { entity, name } => {
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

                EditorEvent::LightChanged { entity, light } => {
                    if let Some(inspector) = &mut self.inspector {
                        if inspector.entity == entity {
                            inspector.light = Some(light.clone())
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
        };
        self.stack.build(ui, &mut ctx);
        self.edit = edit;
        self.platform.prepare_render(ui, window);
        self.end_frame();
    }
}
