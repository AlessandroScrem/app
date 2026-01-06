use core::f32;

use super::*;
use cgmath::{Deg, Rad, num_traits::zero};

use crate::{
    DomainEvent, Globals, camera::Camera, prelude::ui::imgui_layer::DEMO_OPEN, text_fmt,
    timestep::Timestep,
};

pub fn ui_settings(ui: &imgui::Ui, timestep: &Timestep, ctx: &mut UiContext) {
    let camera = &mut ctx.snapshot.camera;
    let globals = &mut ctx.snapshot.globals;
    let hovered_entity = ctx.snapshot.hovered;
    let selected_entity = &ctx.snapshot.selected;
    let adapter_name = &ctx.snapshot.adapter_string;
    let hdrpath = &ctx.snapshot.hdrpath;
    let registry = &ctx.registry;

    ui.window("Settings")
        .size([300.0, 300.0], Condition::FirstUseEver)
        .build(|| {
            if ui.collapsing_header("Parameters", TreeNodeFlags::empty()) {
                let mouse_pos = ui.io().mouse_pos;
                ui.text(format!(
                    "Mouse position: ({:.1},{:.1})",
                    mouse_pos[0], mouse_pos[1]
                ));
                text_fmt!(ui, "ResultGetPixel  : {} ", 0);
                text_fmt!(ui, "Hovered Entity ID: {:?}", hovered_entity,);
                text_fmt!(ui, "Selected Entity ID: {:?}", selected_entity,);
                ui.separator();
                tools::disabled(ui, || {
                    text_fmt!(ui, "NumShaders         : {}", 0);
                    text_fmt!(ui, "NumTextures        : {}", 0);
                    text_fmt!(ui, "NumUniqueTextures  : {}", 0);
                    text_fmt!(ui, "Texture Memory Size: {}", 0);
                    text_fmt!(ui, "Memory Allocations : {}", 0);
                    text_fmt!(ui, "Memory Size        : {}", 0);
                });
            }
            if ui.collapsing_header("Statistics", TreeNodeFlags::DEFAULT_OPEN) {
                text_fmt!(ui, "FPS           : {:?}", timestep.average_fps());
                text_fmt!(ui, "Frametime     : {:?}", timestep.average());
                ui.separator();
                text_fmt!(ui, "Gpu info\n  Adapter:  {}\n  Version:  ", adapter_name);
            }

            tools::disabled(ui, || {
                if ui.collapsing_header("SSAO", TreeNodeFlags::empty()) {}
            });

            if ui.collapsing_header("Toggles", TreeNodeFlags::empty()) {
                unsafe {
                    let mut d_o = DEMO_OPEN;
                    if ui.checkbox("Show demo window", &mut d_o) {
                        DEMO_OPEN = d_o;
                    }

                    if d_o  {
                        ui.show_demo_window(&mut d_o);
                        DEMO_OPEN = d_o;
                    }
                }
                globals.draw_ui(ui);

                ui.separator();
                if let Some(command) = draw_ui_skybox_selector(&ui, hdrpath, registry) {
                    ctx.commands.push_back(command);
                }
            }

            if let Some(command) = camera.draw_ui(ui) {
                ctx.commands.push_back(command)
            }
        });
}

impl Globals {
    fn draw_ui(&mut self, ui: &Ui) -> Option<DomainEvent> {
        let command: Option<DomainEvent> = None;

        const TONEMAP_FILTERS: [&str; 8] = [
            "ACES",
            "Filmic",
            "Lottes",
            "Reinhard",
            "Reinhard2",
            "Uchimura",
            "Uncharted2",
            "Exponential",
        ];

        const DEBUG_CODE: [&str; 11] = [
            "None",
            "Base Color",
            "Normal Texture",
            "Geometry Normal",
            "Geometry Tangent",
            "Geometry Bitangent",
            "Geometry Tangent W",
            "Metallic",
            "Roughness",
            "Occlusion",
            "Emissive",
        ];

        tools::disabled(ui, || {
            let mut mode = false;
            ui.checkbox("Vsync", &mut mode);
        });

        ui.checkbox("Ibl enable", &mut self.ibl_enable);
        ui.checkbox("Skybox enable", &mut self.skybox_enable);
        ui.checkbox("Axis enable", &mut self.axis_enable);
        ui.checkbox("BoundingBox", &mut self.bbox_enable);
        if self.bbox_enable {
            ui.same_line();
            ui.checkbox("BoxAligned", &mut self.bbox_axis_aligned);
        }

        {
            let mut current_item = self.debug_code as usize;
            if ui.combo("Debug Mode", &mut current_item, &DEBUG_CODE, |item| {
                std::borrow::Cow::Borrowed(*item)
            }) {
                self.debug_code = current_item as u32;
            }
        }

        ui.slider_config("Scene Exposure", 0.001, 64.0)
            .flags(SliderFlags::LOGARITHMIC)
            .build(&mut self.exposure);
        ui.slider_config("Ibl Intensity", 0.01, 10_000.0)
            .flags(SliderFlags::LOGARITHMIC)
            .build(&mut self.ibl_intensity);
        ui.separator();

        {
            let mut current_item = self.tonemap_filter as usize;
            if ui.combo("Tonemap", &mut current_item, &TONEMAP_FILTERS, |item| {
                std::borrow::Cow::Borrowed(*item)
            }) {
                self.tonemap_filter = current_item as u32;
            }
        }
        command
    }
}

impl Camera {
    fn draw_ui(&mut self, ui: &Ui) -> Option<DomainEvent> {
        let mut command: Option<DomainEvent> = None;

        ui.text(format!("Position: {:?}", self.get_position()));
        ui.text(format!("FocalPoint: {:?}", self.get_focal_point()));
        ui.text(format!(
            "Yaw/Pitch: {:.1} {:.1}",
            self.get_yaw_pitch().0,
            self.get_yaw_pitch().1
        ));

        if ui.button("Recenter self") {
            command = Some(DomainEvent::RecenterCamera);
        }

        ui.separator();
        let mut fov = Deg::from(self.fov).0;
        if Drag::new("Fov")
            .range(1.0, 179.0)
            .speed(1.0)
            .build(ui, &mut fov)
        {
            self.fov = Rad(fov.to_radians());
        }

        let mut distance = self.get_distance();
        if Drag::new("Distance")
            .range(zero(), f32::MAX)
            .speed(1.0)
            .build(ui, &mut distance)
        {
            self.set_distance(distance);
        }

        let mut near = self.near;
        let mut far = self.far;
        if DragRange::new("Near/Far")
            .range(0.01, f32::MAX)
            .speed(1.0)
            .build(ui, &mut near, &mut far)
        {
            let near = near.max(0.1);
            let far = far.max(near + 0.1);
            self.near = near;
            self.far = far;
        }

        command
    }
}

fn draw_ui_skybox_selector(
    ui: &Ui,
    hdrpath: &std::path::Path,
    registry: &ImGuiTextureRegistry,
) -> Option<DomainEvent> {
    let mut command: Option<_> = None;

    let mut change_skybox = false;
    if let Some(id) = registry.ids.get(hdrpath) {
        let name = hdrpath
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("no name");
        change_skybox = ui.image_button(name, *id, [60.0, 60.0]);
        ui.same_line();
        ui.text(name);
        ui.separator();
    }
    if change_skybox {
        use rfd::FileDialog;
        FileDialog::new()
            .add_filter("hdr", &["hdr"])
            .pick_file()
            .map(|f| command = Some(DomainEvent::ChangeSkybox(f)));
    }
    command
}
