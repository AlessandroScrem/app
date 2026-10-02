use super::ui_commands::UiCommands;
use super::{
    EntityListUi, MenuBarUi, PropertyUi, SettingsUi, UiTextureRegistry, ViewportUi,
};
use crate::editor::EditorCommandClient;
use crate::ui::tools;

use imgui::Ui;
use imgui_winit_support::WinitPlatform;
use winit::event::Event;
use winit::window::Window;

pub struct UiContext<'a> {
    pub commands: &'a EditorCommandClient,
    pub textures: &'a UiTextureRegistry,
    pub material_preview: &'a mut Option<crate::assets::MaterialId>,
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
    fn update(&mut self, commands: &UiCommands);
}

impl Layer for UiStack {
    fn build(&mut self, ui: &Ui, ctx: &mut UiContext) {
        for layer in self.layers.iter_mut() {
            layer.build(ui, ctx);
        }
    }

    fn update(&mut self, commands: &UiCommands) {
        for layer in self.layers.iter_mut() {
            layer.update(commands);
        }
    }
}

pub struct UiLayer {
    context: imgui::Context,
    pub platform: WinitPlatform,
    ini_loaded: bool,
    timestep: crate::timestep::Timestep,
    stack: UiStack,
    commands: UiCommands,
    material_preview: Option<crate::assets::MaterialId>,
}

impl UiLayer {
    pub fn new(
        window: &Window,
        mut context: imgui::Context,
        adapter_string: String,
        connection: crate::editor::EditorConnection,
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
        ui.push(MenuBarUi::default());
        ui.push(EntityListUi::default());
        ui.push(PropertyUi::default());
        ui.push(SettingsUi::new(adapter_string));

        Self {
            context,
            platform,
            ini_loaded: false,
            timestep: crate::timestep::Timestep::new(),
            stack: ui,
            commands: UiCommands::new(connection),
            material_preview: None,
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

    pub fn build(&mut self, window: &Window, textures: &UiTextureRegistry) {
        self.commands.process();
        self.begin_frame(window);
        self.stack.update(&self.commands);

        let ui = self.context.frame();
        ui.dockspace_over_main_viewport();

        let command_client = self.commands.command_client();
        self.material_preview = None;
        let mut ctx = UiContext {
            commands: command_client,
            textures,
            material_preview: &mut self.material_preview,
        };

        self.stack.build(ui, &mut ctx);
        self.platform.prepare_render(ui, window);
        self.end_frame();
    }

    pub fn material_preview(&self) -> Option<crate::assets::MaterialId> {
        self.material_preview
    }
}
