use std::{collections::VecDeque};

use super::*;
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::WinitPlatform;
use legion::Entity;
use winit::window::Window;

use crate::{DomainEvent, Globals, UiComponentView, camera::Camera, timestep::Timestep};

pub struct UiContext<'a, 'b> {
    pub snapshot: &'a mut Snapshot<'b>,
    pub registry: &'a ImGuiTextureRegistry,
    pub commands: VecDeque<DomainEvent>,
}

pub struct Snapshot<'a> {
    pub camera: &'a mut Camera,
    pub globals: &'a mut Globals,
    pub root_nodes: &'a RootNodes,
    pub lights_nodes: &'a RootNodes,
    pub comp_view: &'a mut UiComponentView,
    pub selected: &'a mut Option<Entity>,
    pub hovered: Option<Entity>,
}

pub struct HierarchyNode {
    pub name: String,
    pub parent: Option<Entity>,
    pub entity: Entity,
    pub children: Vec<HierarchyNode>,
}

#[derive(Default)]
pub struct RootNodes {
    pub nodes: Vec<HierarchyNode>,
}

#[derive(Default)]
pub struct RootSnapshot {
    pub  root_nodes: RootNodes,
    pub lights_nodes: RootNodes,
}

pub struct ImguiLayer {
    pub context: imgui::Context,
    pub platform: WinitPlatform,
    pub renderer: Renderer,
    pub registry: ImGuiTextureRegistry,
    pub last_cursor: Option<MouseCursor>,
    ini_loaded: bool,
    timestep: Timestep,
}

pub static mut DEMO_OPEN: bool = false;

impl ImguiLayer {
    pub fn new(
        window: &Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self {
        let mut context = imgui::Context::create();

        let io = context.io_mut();
        io.config_flags.insert(ConfigFlags::DOCKING_ENABLE);
        io.config_flags.insert(ConfigFlags::VIEWPORTS_ENABLE);

        tools::set_dark_theme_colors(context.style_mut());

        let mut platform = WinitPlatform::new(&mut context);
        let hidpi_factor = window.scale_factor();

        platform.attach_window(
            context.io_mut(),
            window,
            imgui_winit_support::HiDpiMode::Default,
        );

        context.set_ini_filename(None);

        let font_size = (9.0 * hidpi_factor) as f32;

        context.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);

        let renderer_config = RendererConfig {
            texture_format: format,
            ..Default::default()
        };
        let renderer = Renderer::new(&mut context, &device, &queue, renderer_config);

        let registry = ImGuiTextureRegistry::new();
        let timestep = Timestep::new();

        Self {
            context,
            platform,
            renderer,
            registry,
            last_cursor: None,
            ini_loaded: false,
            timestep,
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

    pub fn update_ui(&mut self, window: &Window, snapshot: &mut Snapshot) ->VecDeque<DomainEvent>{
        self.timestep.update();
        let delta_s = self.timestep.delta();

        self.context.io_mut().update_delta_time(delta_s);

        self.platform
            .prepare_frame(self.context.io_mut(), &window)
            .expect("failed_to prepare frame");

        let ui = self.context.frame();
        let commands = {
            let mut ctx = UiContext {
                snapshot,
                registry: &self.registry,
                commands: VecDeque::new(),
            };
            ui.dockspace_over_main_viewport();

            windows::draw_window_settings(ui, &self.timestep, &mut ctx);
            windows::draw_window_entities(ui, &mut ctx);
            windows::draw_window_properties(ui, &mut ctx);

            if unsafe { DEMO_OPEN } {
                ui.show_demo_window(&mut true);
            }

            windows::draw_debug_texture(ui, &ctx);
            ctx.commands
        };

        // update window cursor state (icon)
        if self.last_cursor != ui.mouse_cursor() {
            self.last_cursor = ui.mouse_cursor();
            self.platform.prepare_render(ui, window);
        };

        self.load_ini_if_needed();

        commands
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
