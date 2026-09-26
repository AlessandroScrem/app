use super::*;
use crate::editor::EditorCommandClient;
use super::ui_commands::UiCommands;

use imgui::Ui;
use imgui_winit_support::WinitPlatform;
use winit::event::Event;
use winit::window::Window;

pub struct UiContext<'a> {
    pub commands: &'a EditorCommandClient,
    pub hierarchy: Option<&'a crate::editor::HierarchyData>,
    pub selection: &'a [crate::editor::EntityId],
    pub inspector: Option<&'a crate::editor::InspectorData>,
    pub settings: Option<&'a crate::editor::EditorSettingsData>,
    pub statistics: Option<&'a crate::editor::EditorStatisticsData>,
    pub edit: &'a mut Option<crate::editor::EditorEdit<crate::editor::EntityId, crate::editor::EditValue>>,
    pub scene_settings: &'a crate::editor::SceneSettingsData,
    pub adapter_string: &'a String,
}

struct UiStack { layers: Vec<Box<dyn Layer>> }

impl UiStack {
    fn new() -> Self { Self { layers: Vec::new() } }
    fn push<L: Layer + 'static>(&mut self, layer: L) { self.layers.push(Box::new(layer)); }
}

pub trait Layer { fn build(&mut self, ui: &Ui, ctx: &mut UiContext); }

impl Layer for UiStack {
    fn build(&mut self, ui: &Ui, ctx: &mut UiContext) {
        for layer in self.layers.iter_mut() { layer.build(ui, ctx); }
    }
}

pub struct UiLayer {
    context: imgui::Context,
    pub platform: WinitPlatform,
    ini_loaded: bool,
    timestep: crate::timestep::Timestep,
    stack: UiStack,
    adapter_string: String,
    commands: UiCommands,
}

impl UiLayer {
    pub fn new(window: &Window, mut context: imgui::Context, adapter_string: String, connection: crate::editor::EditorConnection) -> Self {
        tools::set_dark_theme_colors(context.style_mut());
        let io = context.io_mut();
        io.config_flags.insert(imgui::ConfigFlags::DOCKING_ENABLE);
        io.config_flags.insert(imgui::ConfigFlags::VIEWPORTS_ENABLE);
        context.set_ini_filename(None);
        let mut platform = WinitPlatform::new(&mut context);
        platform.attach_window(context.io_mut(), window, imgui_winit_support::HiDpiMode::Default);
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
            commands: UiCommands::new(connection),
        }
    }

    pub fn want_capture_mouse(&self) -> bool { self.context.io().want_capture_mouse }

    pub fn handle_event<T>(&mut self, window: &Window, event: &Event<T>) {
        self.platform.handle_event::<T>(self.context.io_mut(), window, event);
    }

    pub fn get_draw_data(&mut self) -> &imgui::DrawData { self.context.render() }

    fn begin_frame(&mut self, window: &Window) {
        self.timestep.update();
        self.context.io_mut().update_delta_time(self.timestep.delta());
        self.platform.prepare_frame(self.context.io_mut(), window).expect("failed to prepare frame");
    }

    fn end_frame(&mut self) {
        if !self.ini_loaded {
            self.context.set_ini_filename(Some("imgui.ini".into()));
            if let Ok(content) = std::fs::read_to_string("imgui.ini") { self.context.load_ini_settings(&content); }
            self.ini_loaded = true;
        }
    }

    pub fn build(&mut self, window: &Window) {
        self.commands.process();
        self.begin_frame(window);
        let ui = self.context.frame();
        ui.dockspace_over_main_viewport();

        let hierarchy = self.commands.hierarchy();
        let selection = self.commands.selection();
        let inspector = self.commands.inspector();
        let settings = self.commands.settings();
        let statistics = self.commands.statistics();
        let scene_settings = self.commands.scene_settings();
        let commands = self.commands.commands();
        let mut edit = self.commands.take_edit();

        let mut ctx = UiContext {
            commands, hierarchy, selection, inspector, settings, statistics,
            edit: &mut edit, scene_settings, adapter_string: &self.adapter_string,
        };
        self.stack.build(ui, &mut ctx);
        self.commands.set_edit(edit);
        self.platform.prepare_render(ui, window);
        self.end_frame();
    }
}
