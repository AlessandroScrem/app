use std::collections::VecDeque;

use super::*;
use imgui::*;

use imgui_winit_support::WinitPlatform;
use winit::window::Window;

use crate::gpu::GpuCache;
use crate::renderer::imgui_renderer::ImguiRender;
use crate::renderer::scene_renderer::FrameStats;
use crate::timestep::Timestep;

pub struct UiRuntimeContext<'a> {
    pub window: &'a Window,
    pub uilayer: &'a mut UiLayer,
    pub imgui_render: &'a ImguiRender,
    pub gpu_cache: &'a GpuCache,
    pub frame_stats: FrameStats,
}

pub struct UiContext<'a> {
    pub snapshot: &'a UiSnapshot<'a>,
    pub write: UiWriteModel,
    pub timestep: Timestep,
    pub adapter_string: String,
}

pub struct UiWriteModel {
    pub commands: VecDeque<DomainEvent>,
}

impl UiWriteModel {
    pub fn new() -> Self {
        Self {
            commands: VecDeque::new(),
        }
    }

    pub fn push(&mut self, cmd: DomainEvent) {
        self.commands.push_back(cmd);
    }
}

pub struct UiLayer {
    context: imgui::Context,
    pub platform: WinitPlatform,
    ini_loaded: bool,
    timestep: Timestep,
    stack: UiStack,
    adapter_string: String,
}

struct UiStack {
    layers: Vec<Box<dyn Layer>>,
}
impl UiStack {
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    pub fn push<L: Layer + 'static>(&mut self, layer: L) {
        self.layers.push(Box::new(layer));
    }
}

pub trait Layer {
    fn build(&mut self, ui: &Ui, ui_context: &mut UiContext);
}

impl Layer for UiStack {
    fn build(&mut self, ui: &Ui, ui_context: &mut UiContext) {
        for layer in self.layers.iter_mut() {
            layer.build(ui, ui_context);
        }
    }
}

impl UiLayer {
    pub fn new(window: &Window, mut context: imgui::Context, adapter_string: String) -> Self {
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

        let timestep = Timestep::new();

        let mut ui = UiStack::new();
        ui.push(MenuBarUi {});
        ui.push(SettimgsUi::default());
        ui.push(EntityListUi {});
        ui.push(PropertyUi {});
        ui.push(DebugUi {});

        Self {
            context,
            platform,
            ini_loaded: false,
            timestep,
            stack: ui,
            adapter_string,
        }
    }

    // wokaround to avoid crash:
    // load ini after creating 1st frame.
    fn load_ini_if_needed(&mut self) {
        if self.ini_loaded {
            return;
        }

        self.context.set_ini_filename(Some("imgui.ini".into()));

        if let Ok(ini_content) = std::fs::read_to_string("imgui.ini") {
            self.context.load_ini_settings(&ini_content);
        }

        self.ini_loaded = true;
    }

    pub fn want_capture_mouse(&self) -> bool {
        self.context.io().want_capture_mouse
    }

    pub fn handle_event(&mut self, window: &Window, event: &winit::event::Event<()>) {
        self.platform
            .handle_event::<()>(self.context.io_mut(), window, &event);
    }

    pub fn get_draw_data(&mut self) -> &imgui::DrawData {
        self.context.render()
    }

    fn begin_frame(&mut self, window: &Window) {
        self.timestep.update();
        let delta_s = self.timestep.delta();

        let io = self.context.io_mut();
        io.update_delta_time(delta_s);

        self.platform
            .prepare_frame(self.context.io_mut(), &window)
            .expect("failed_to prepare frame");
    }

    fn end_frame(&mut self) {
        self.load_ini_if_needed();
    }

    pub fn build(&mut self, window: &Window, snapshot: UiSnapshot) -> VecDeque<DomainEvent> {
        let mut ctx = UiContext {
            snapshot: &snapshot,
            write: UiWriteModel::new(),
            timestep: self.timestep.clone(),
            adapter_string: self.adapter_string.clone(),
        };

        self.begin_frame(window);

        {
            let ui = self.context.frame();

            ui.dockspace_over_main_viewport();

            self.stack.build(ui, &mut ctx);

            self.platform.prepare_render(ui, window);
        };

        self.end_frame();

        ctx.write.commands
    }
}

#[cfg(test)]
mod tests {
    use imgui::{ConfigFlags, Context};
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn should_imgui_load_ini() {
        let mut imgui = Context::create();
        imgui
            .io_mut()
            .config_flags
            .insert(ConfigFlags::DOCKING_ENABLE);

        // --- caricamento manuale ---
        let path = PathBuf::from("imgui.ini");
        if let Ok(s) = fs::read_to_string(&path) {
            imgui.load_ini_settings(&s);
        }

        let mut ini_data = String::new();
        imgui.save_ini_settings(&mut ini_data);
        fs::write(path, ini_data).unwrap();
    }
}
