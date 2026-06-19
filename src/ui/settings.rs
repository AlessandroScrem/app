use super::*;

use imgui::*;
use prelude::math::*;

use crate::renderer::GpuResourceStats;
use crate::{DomainEvent, Globals, assets::ResourceStats, camera::Camera, text_fmt};

use std::fmt;

impl fmt::Display for ResourceStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const BYTES_TO_MB: f32 = 1.0 / (1024.0 * 1024.0);
        write!(
            f,
            "{} total | {} shared | {:.2} MB",
            self.count,
            self.shared,
            self.estimated_bytes as f32 * BYTES_TO_MB
        )
    }
}

impl fmt::Display for GpuResourceStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const BYTES_TO_MB: f32 = 1.0 / (1024.0 * 1024.0);
        write!(
            f,
            "{} total | {:.2} MB",
            self.count,
            self.estimated_bytes as f32 * BYTES_TO_MB
        )
    }
}

impl fmt::Display for renderer::framebuilder::DrawStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} calls | {} instances",
            self.draw_calls, self.instances
        )
    }
}

#[derive(Default)]
pub struct SettimgsUi {
    demo_open: bool,
}

impl Layer for SettimgsUi {
    fn build(&mut self, ui: &Ui, ctx: &mut UiContext) {
        let camera = &ctx.snapshot.camera;
        let globals = &ctx.snapshot.globals;
        let hovered_entity = ctx.snapshot.hovered;
        let selected_entity = &ctx.snapshot.selected;
        let adapter_name = &ctx.adapter_string;
        let hdr_texture_id = ctx.snapshot.hdr_id;
        let timestep = &ctx.timestep;
        let texture_resolver = &ctx.snapshot.texture_resolver;
        let gpu_counters = &ctx.snapshot.gpu_counters;
        let root_nodes = ctx.snapshot.root_snapshot.root_nodes.nodes.len();
        let render_stats = &ctx.snapshot.render_stats;
        let opaque_stats = render_stats.frame.opaque;
        let transmission_stats = render_stats.frame.transmission;

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
                        text_fmt!(ui, "Asset Textures     : {}", render_stats.texture);
                        text_fmt!(ui, "Asset Meshes       : {}", render_stats.mesh);
                        text_fmt!(ui, "Asset Materials    : {}", render_stats.material);
                        text_fmt!(ui, "Gpu Textures       : {}", gpu_counters.textures);
                        text_fmt!(ui, "Gpu Materials      : {}", gpu_counters.materials);
                        text_fmt!(ui, "Gpu Meshes         : {}", gpu_counters.meshes);
                        text_fmt!(ui, "Gpu int Buffers    : {}", 0);
                        text_fmt!(ui, "Gpu VB             : {}", 0);
                        text_fmt!(ui, "Gpu FB             : {}", 0);
                        text_fmt!(ui, "Gpu Mem            : {}", 0);
                        text_fmt!(ui, "GPU Shaders        : {}", 0);
                        text_fmt!(ui, "Opaque             : {}", opaque_stats);
                        text_fmt!(ui, "Transmission       : {}", transmission_stats);
                        text_fmt!(ui, "RootNodes          : {}", root_nodes);
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
                    ui.checkbox("Show demo window", &mut self.demo_open);
                    if self.demo_open {
                        ui.show_demo_window(&mut self.demo_open);
                    }

                    if let Some(command) = globals.draw_ui(ui) {
                        ctx.write.push(command);
                    }

                    ui.separator();
                    if let Some(hdr_texture_id) = hdr_texture_id {
                        if let Some(command) = draw_ui_skybox_selector(
                            &ui,
                            texture_resolver.resolve(UiTexture::Engine(hdr_texture_id)),
                        ) {
                            ctx.write.push(command);
                        }
                    }
                }

                if let Some(command) = camera.draw_ui(ui) {
                    ctx.write.push(command)
                }
            });
    }
}

impl Globals {
    fn draw_ui(&self, ui: &Ui) -> Option<DomainEvent> {
        let mut command: Option<DomainEvent> = None;

        const TONEMAP_FILTERS: [&str; 9] = [
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

        const DEBUG_CODE: [&str; 18] = [
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
            "ShennColor",
            "ShennRoughness",
        ];

        tools::disabled(ui, || {
            let mut mode = false;
            ui.checkbox("Vsync", &mut mode);
        });

        let mut light_enable = self.light_enable;
        let mut ibl_enable = self.ibl_enable;
        let mut skybox_enable = self.skybox_enable;
        let mut axis_enable = self.axis_enable;
        let mut bbox_enable = self.bbox_enable;
        let mut mips_cs = self.mips_cs;
        let mut env_rotation = self.env_rotation;

        use GlobalEvent::*;
        if ui.checkbox("Mips with CS", &mut mips_cs) {
            command = Some(DomainEvent::Global(MipsCsEnable(mips_cs)));
        }
        if ui.checkbox("light enable", &mut light_enable) {
            command = Some(DomainEvent::Global(LightEnable(light_enable)));
        }
        if ui.checkbox("Ibl enable", &mut ibl_enable) {
            command = Some(DomainEvent::Global(IblEnable(ibl_enable)));
        }
        if ui.checkbox("Skybox enable", &mut skybox_enable) {
            command = Some(DomainEvent::Global(SkyboxEnable(skybox_enable)));
        }
        if self.skybox_enable {
            let mut skybox_enable_blur = self.skybox_enable_blur;
            ui.same_line();
            if ui.checkbox("Blur", &mut skybox_enable_blur) {
                command = Some(DomainEvent::Global(SkyboxEnableBlur(skybox_enable_blur)));
            }
        }
        if ui
            .slider_config("Env rotation", 0.0, 360.0)
            .build(&mut env_rotation)
        {
            if !ui.io().key_shift {
                ui.tooltip_text("Press shift to disable snap");

                let snap = 90.0;
                let threshold = 25.0;
                let nearest = (env_rotation / snap).round() * snap;
                if (env_rotation - nearest).abs() < threshold {
                    env_rotation = nearest;
                }
                if !ui.is_item_active() {
                    env_rotation = nearest;
                }
            }

            command = Some(DomainEvent::Global(EnvRotation(env_rotation)));
        }

        if ui.checkbox("Axis enable", &mut axis_enable) {
            command = Some(DomainEvent::Global(AxisEnable(axis_enable)));
        }
        if ui.checkbox("BoundingBox", &mut bbox_enable) {
            command = Some(DomainEvent::Global(BboxEnable(bbox_enable)));
        }
        if self.bbox_enable {
            let mut bbox_axis_aligned = self.bbox_axis_aligned;
            ui.same_line();
            if ui.checkbox("BoxAligned", &mut bbox_axis_aligned) {
                command = Some(DomainEvent::Global(BboxAxisAligned(bbox_axis_aligned)));
            }
        }

        {
            let mut current_item = self.debug_code as usize;
            if ui.combo("Debug Mode", &mut current_item, &DEBUG_CODE, |item| {
                std::borrow::Cow::Borrowed(*item)
            }) {
                command = Some(DomainEvent::Global(DebugCode(current_item as u32)));
            }
        }

        let mut exposure = self.exposure;
        let mut ibl_intensity = self.ibl_intensity;
        if ui
            .slider_config("Scene Exposure", 0.001, 64.0)
            .flags(SliderFlags::LOGARITHMIC)
            .build(&mut exposure)
        {
            command = Some(DomainEvent::Global(Exposure(exposure)))
        }
        if ui
            .slider_config("Ibl Intensity", 0.01, 10_000.0)
            .flags(SliderFlags::LOGARITHMIC)
            .build(&mut ibl_intensity)
        {
            command = Some(DomainEvent::Global(IblIntensity(ibl_intensity)))
        }
        ui.separator();

        {
            let mut current_item = self.tonemap_filter as usize;
            if ui.combo("Tonemap", &mut current_item, &TONEMAP_FILTERS, |item| {
                std::borrow::Cow::Borrowed(*item)
            }) {
                // self.tonemap_filter = current_item as u32;
                command = Some(DomainEvent::Global(TonemapFilter(current_item as u32)))
            }
        }
        command
    }
}

impl Camera {
    fn draw_ui(&self, ui: &Ui) -> Option<DomainEvent> {
        use CameraEvent::*;
        let mut command: Option<DomainEvent> = None;

        ui.text(format!("Position: {:?}", self.get_position()));
        ui.text(format!("FocalPoint: {:?}", self.get_focal_point()));
        ui.text(format!(
            "Yaw/Pitch: {:.1} {:.1}",
            self.get_yaw_pitch().0,
            self.get_yaw_pitch().1
        ));

        if ui.button("Recenter self") {
            command = Some(DomainEvent::Camera(RecenterCamera));
        }

        ui.separator();
        let mut fov = Deg::from(self.get_fov()).0;
        if Drag::new("Fov")
            .range(1.0, 179.0)
            .speed(1.0)
            .build(ui, &mut fov)
        {
            // self.fov = Rad(fov.to_radians());
            command = Some(DomainEvent::Camera(CameraFov(Rad(fov.to_radians()))));
        }

        let mut distance = self.get_distance();
        if Drag::new("Distance")
            .range(zero(), f32::MAX)
            .speed(1.0)
            .build(ui, &mut distance)
        {
            // self.set_distance(distance);
            command = Some(DomainEvent::Camera(CameraDistance(distance)));
        }

        let (mut near, mut far) = self.get_near_far();
        if DragRange::new("Near/Far")
            .range(0.01, f32::MAX)
            .speed(1.0)
            .build(ui, &mut near, &mut far)
        {
            let near = near.max(0.1);
            let far = far.max(near + 0.1);
            // self.near = near;
            // self.far = far;
            command = Some(DomainEvent::Camera(CameraNearFar((near, far))));
        }

        command
    }
}

fn draw_ui_skybox_selector(ui: &Ui, hdr_texture_id: Option<TextureId>) -> Option<DomainEvent> {
    let mut command: Option<_> = None;

    let mut change_skybox = false;
    if let Some(id) = hdr_texture_id {
        change_skybox = ui.image_button("##name", id, [60.0, 60.0]);
        ui.same_line();
        ui.text("HdrTexture");
        ui.separator();
    }
    if change_skybox {
        use rfd::FileDialog;
        FileDialog::new()
            .add_filter("hdr", &["hdr"])
            .pick_file()
            .map(|f| command = Some(DomainEvent::Assets(AssetEvent::ChangeSkybox(f))));
    }
    command
}
