use super::title_bar::TopBarUi;
use super::ui_commands::UiCommands;
use super::{EntityListUi, PropertyUi, SettingsUi, UiTextures, ViewportUi};
use crate::editor::EditorCommandClient;
use crate::ui::tools;
use crate::winit_bridge::{WindowHandle, WinitUiPlatform};
use imgui::Ui;

pub struct UiContext<'a> {
    pub commands: &'a EditorCommandClient,
    pub textures: &'a UiTextures,
    pub material_preview: &'a mut Option<crate::assets::MaterialId>,
    pub window_action: &'a mut Option<WindowAction>,
    pub maximize_hovered: bool,
    pub window_title: &'a str,
}

pub struct UiOutput {
    pub material_preview: Option<crate::assets::MaterialId>,
    pub window_action: Option<WindowAction>,
}

pub enum WindowAction {
    Drag,
    Minimize,
    ToggleMaximize,
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
    platform: WinitUiPlatform,
    ini_loaded: bool,
    timestep: crate::timestep::Timestep,
    stack: UiStack,
    commands: UiCommands,
    window_title: String,
}

impl UiLayer {
    pub fn new(
        window: &WindowHandle,
        mut context: imgui::Context,
        adapter_string: String,
        connection: crate::editor::EditorConnection,
    ) -> Self {
        tools::set_dark_theme_colors(context.style_mut());
        let io = context.io_mut();
        io.config_flags.insert(imgui::ConfigFlags::DOCKING_ENABLE);
        io.config_flags
            .insert(imgui::ConfigFlags::VIEWPORTS_ENABLE);
        context.set_ini_filename(None);

        let platform = WinitUiPlatform::new(&mut context, window);

        let mut ui = UiStack::new();
        ui.push(ViewportUi::default());
        ui.push(EntityListUi::default());
        ui.push(PropertyUi::default());
        ui.push(SettingsUi::new(adapter_string));
        ui.push(TopBarUi::default());

        Self {
            context,
            platform,
            ini_loaded: false,
            timestep: crate::timestep::Timestep::new(),
            stack: ui,
            commands: UiCommands::new(connection),
            window_title: "App".to_owned(),
        }
    }

    pub fn set_window_title(&mut self, title: &str) {
        self.window_title.clear();
        self.window_title.push_str(title);
    }

    pub fn want_capture_mouse(&self) -> bool {
        self.context.io().want_capture_mouse
    }

    pub fn handle_event(
        &mut self,
        window: &WindowHandle,
        event: &crate::winit_bridge::WinitEvent,
    ) {
        self.platform
            .handle_event(&mut self.context, window, event);
    }

    pub fn get_draw_data(&mut self) -> &imgui::DrawData {
        self.context.render()
    }

    fn begin_frame(&mut self, window: &WindowHandle) {
        self.timestep.update();
        self.context
            .io_mut()
            .update_delta_time(self.timestep.delta());
        self.platform.prepare_frame(&mut self.context, window);
    }

    fn set_main_viewport_work_area() {
        const TOP_BAR_HEIGHT: f32 = 36.0;

        unsafe {
            let viewport = imgui::sys::igGetMainViewport();
            (*viewport).WorkPos.y = (*viewport).Pos.y + TOP_BAR_HEIGHT;
            (*viewport).WorkSize.y = ((*viewport).Size.y - TOP_BAR_HEIGHT).max(0.0);
        }
    }

    fn end_frame(&mut self) {
        if !self.ini_loaded {
            const INI_FILE: &str = "imgui_custom_titlebar.ini";
            self.context.set_ini_filename(Some(INI_FILE.into()));
            if let Ok(content) = std::fs::read_to_string(INI_FILE) {
                self.context.load_ini_settings(&content);
            }
            self.ini_loaded = true;
        }
    }

    pub fn build(&mut self, window: &WindowHandle, textures: &UiTextures) -> UiOutput {
        self.commands.process();
        self.begin_frame(window);
        self.stack.update(&self.commands);

        let ui = self.context.frame();
        Self::set_main_viewport_work_area();
        ui.dockspace_over_main_viewport();

        let command_client = self.commands.command_client();
        let mut output = UiOutput {
            material_preview: None,
            window_action: None,
        };
        let mut ctx = UiContext {
            commands: command_client,
            textures,
            material_preview: &mut output.material_preview,
            window_action: &mut output.window_action,
            maximize_hovered: crate::winit_bridge::maximize_button_hovered(window),
            window_title: &self.window_title,
        };

        self.stack.build(ui, &mut ctx);
        self.platform.prepare_render(ui, window);
        self.end_frame();

        output
    }
}
