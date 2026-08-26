use super::ui_layer::{Layer, UiContext};
use crate::editor::{EditorCommand, EditorSettingsData};
use imgui::{Drag, SliderFlags, TreeNodeFlags, Ui};

#[derive(Default)]
pub struct SettingsUi {
    demo_open: bool,
}

impl Layer for SettingsUi {
    fn build(&mut self, ui: &Ui, ctx: &mut UiContext) {
        let Some(settings) = ctx.settings else {
            ui.window("Settings")
                .size([320.0, 300.0], imgui::Condition::FirstUseEver)
                .build(|| ui.text("Loading settings..."));
            return;
        };
        ui.window("Settings")
            .size([320.0, 500.0], imgui::Condition::FirstUseEver)
            .build(|| self.draw(ui, ctx, settings));
    }
}

impl SettingsUi {
    fn draw(&mut self, ui: &Ui, ctx: &mut UiContext, settings: &EditorSettingsData) {
        if ui.collapsing_header("Statistics", TreeNodeFlags::DEFAULT_OPEN) {
            if let Some(stats) = ctx.statistics {
                ui.text(format!("FPS: {:.1}", stats.fps));
                ui.text(format!("Frametime: {:.2} ms", stats.frametime * 1000.0));
                ui.text(format!("Root nodes: {}", stats.root_nodes));
                ui.text(format!(
                    "Opaque: {} calls | {} instances",
                    stats.opaque_draw_calls, stats.opaque_instances
                ));
                ui.text(format!(
                    "Transmission: {} calls | {} instances",
                    stats.transmission_draw_calls, stats.transmission_instances
                ));
                ui.text(format!("Adapter: {}", stats.adapter_name));
            } else {
                ui.text("Collecting statistics...");
            }
        }
        if ui.collapsing_header("Toggles", TreeNodeFlags::DEFAULT_OPEN) {
            toggle(
                ui,
                "Mips with CS",
                settings.mips_cp,
                ctx,
                EditorCommand::SetMipsWithCompute,
            );
            toggle(
                ui,
                "Light",
                settings.light_enable,
                ctx,
                EditorCommand::SetLightEnable,
            );
            toggle(
                ui,
                "IBL",
                settings.ibl_enable,
                ctx,
                EditorCommand::SetIblEnable,
            );
            toggle(
                ui,
                "Skybox",
                settings.skybox_enable,
                ctx,
                EditorCommand::SetSkyboxEnable,
            );
            toggle(
                ui,
                "Skybox blur",
                settings.skybox_enable_blur,
                ctx,
                EditorCommand::SetSkyboxBlur,
            );
            toggle(
                ui,
                "Axis",
                settings.axis_enable,
                ctx,
                EditorCommand::SetAxisEnable,
            );
            toggle(
                ui,
                "Bounding box",
                settings.bbox_enable,
                ctx,
                EditorCommand::SetBoundingBoxEnable,
            );
            if settings.bbox_enable {
                toggle(
                    ui,
                    "Box aligned",
                    settings.bbox_axis_aligned,
                    ctx,
                    EditorCommand::SetBoundingBoxAxisAligned,
                );
            }
            let mut exposure = settings.exposure;
            if ui
                .slider_config("Scene Exposure", 0.001, 64.0)
                .flags(SliderFlags::LOGARITHMIC)
                .build(&mut exposure)
            {
                ctx.connection
                    .commands
                    .send(EditorCommand::SetExposure(exposure));
            }
            let mut ibl_intensity = settings.ibl_intensity;
            if ui
                .slider_config("IBL Intensity", 0.01, 10_000.0)
                .flags(SliderFlags::LOGARITHMIC)
                .build(&mut ibl_intensity)
            {
                ctx.connection
                    .commands
                    .send(EditorCommand::SetIblIntensity(ibl_intensity));
            }
            let mut env_rotation = settings.env_rotation;
            if ui
                .slider_config("Env rotation", 0.0, 360.0)
                .build(&mut env_rotation)
            {
                ctx.connection
                    .commands
                    .send(EditorCommand::SetEnvironmentRotation(env_rotation));
            }
            const DEBUG: [&str; 18] = [
                "None",
                "TextureCoords0",
                "TextureCoords1",
                "Base Color",
                "Normal Texture",
                "Geometry Normal",
                "Geometry Tangent",
                "Geometry Bitangent",
                "Geometry Tangent W",
                "ShadingNormal",
                "Metallic",
                "Roughness",
                "Emissive",
                "Occlusion",
                "Transmission",
                "VolumeThickness",
                "SheenColor",
                "SheenRoughness",
            ];
            let mut debug = settings.debug_code as usize;
            if ui.combo("Debug Mode", &mut debug, &DEBUG, |item| {
                std::borrow::Cow::Borrowed(*item)
            }) {
                ctx.connection
                    .commands
                    .send(EditorCommand::SetDebugCode(debug as u32));
            }
            const TONEMAP: [&str; 9] = [
                "Khronos PBR Neutral",
                "ACES",
                "Filmic",
                "Lottes",
                "Reinhard",
                "Reinhard2",
                "Uchimura",
                "Uncharted2",
                "Exponential",
            ];
            let mut tonemap = settings.tonemap_filter as usize;
            if ui.combo("Tonemap", &mut tonemap, &TONEMAP, |item| {
                std::borrow::Cow::Borrowed(*item)
            }) {
                ctx.connection
                    .commands
                    .send(EditorCommand::SetTonemap(tonemap as u32));
            }
            if ui.button("Add IBL") {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("hdr", &["hdr"])
                    .pick_file()
                {
                    ctx.connection.commands.send(EditorCommand::AddIbl { path });
                }
            }
            ui.checkbox("Show demo window", &mut self.demo_open);
            if self.demo_open {
                ui.show_demo_window(&mut self.demo_open);
            }
        }
        if ui.collapsing_header("Camera", TreeNodeFlags::DEFAULT_OPEN) {
            ui.text(format!("FOV: {:.1}", settings.camera_fov));
            if ui.button("Recenter") {
                ctx.connection.commands.send(EditorCommand::RecenterCamera);
            }
            let mut fov = settings.camera_fov;
            if Drag::new("FOV")
                .range(1.0, 179.0)
                .speed(1.0)
                .build(ui, &mut fov)
            {
                ctx.connection
                    .commands
                    .send(EditorCommand::SetCameraFov(fov));
            }
            let mut distance = settings.camera_distance;
            if Drag::new("Distance")
                .range(0.0, f32::MAX)
                .speed(1.0)
                .build(ui, &mut distance)
            {
                ctx.connection
                    .commands
                    .send(EditorCommand::SetCameraDistance(distance));
            }
            let mut near = settings.camera_near;
            let mut far = settings.camera_far;
            if Drag::new("Near")
                .range(0.01, f32::MAX)
                .speed(0.1)
                .build(ui, &mut near)
            {
                ctx.connection
                    .commands
                    .send(EditorCommand::SetCameraNearFar { near, far });
            }
            if Drag::new("Far")
                .range(0.1, f32::MAX)
                .speed(1.0)
                .build(ui, &mut far)
            {
                ctx.connection
                    .commands
                    .send(EditorCommand::SetCameraNearFar { near, far });
            }
        }
    }
}

fn toggle<F>(ui: &Ui, label: &str, value: bool, ctx: &mut UiContext, make: F)
where
    F: FnOnce(bool) -> EditorCommand,
{
    let mut value = value;
    if ui.checkbox(label, &mut value) {
        ctx.connection.commands.send(make(value));
    }
}
