use super::*;
use cgmath::{Deg, Rad};
use legion::Resources;

use crate::{
    assets::texture_manager::TextureManager,
    camera::Camera,
    picking::PickObject,
    renderer::{GPUResourceManager, skybox_manager::SkyboxManager},
    text_fmt,
    timestep::Timestep,
};

pub fn draw_window_settings(timestep: &Timestep, ctx: &mut InspectorContext) {
    let ui = ctx.ui;

    ctx.ui
        .window("Settings")
        .size([300.0, 300.0], Condition::FirstUseEver)
        .build(|| {
            draw_ui_parameters(ctx);
            draw_ui_statistics(ctx, &timestep);
            draw_ui_toggles(ctx);
            tools::disabled(ui, || {
                if ui.collapsing_header("SSAO", TreeNodeFlags::empty()) {}
            });
            draw_ui_camera(ctx);
        });
}

fn draw_ui_parameters(ctx: &InspectorContext) {
    let ui = ctx.ui;
    if ui.collapsing_header("Parameters", TreeNodeFlags::empty()) {
        let pick_object = ctx.resources.get::<PickObject>().unwrap();

        let mouse_pos = ui.io().mouse_pos;
        ui.text(format!(
            "Mouse position: ({:.1},{:.1})",
            mouse_pos[0], mouse_pos[1]
        ));
        text_fmt!(ui, "ResultGetPixel  : {} ", 0);
        let hovered_entity_name = "Noname";
        let selected_entity_name = "Noname";
        let hovered_entity = pick_object.hovered;
        let selected_entity = pick_object.selected;
        text_fmt!(
            ui,
            "Hovered Entity  : {} ID: {:?}",
            hovered_entity_name,
            hovered_entity,
        );
        text_fmt!(
            ui,
            "Selected Entity : {} ID: {:?}",
            selected_entity_name,
            selected_entity,
        );
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
}

fn draw_ui_statistics(ctx: &InspectorContext, timestep: &Timestep) {
    let ui = ctx.ui;
    if ui.collapsing_header("Statistics", TreeNodeFlags::DEFAULT_OPEN) {
        let adapter_name = ctx
            .resources
            .get::<wgpu::Adapter>()
            .unwrap()
            .get_info()
            .name;

        text_fmt!(ui, "FPS           : {:?}", timestep.average_fps());
        text_fmt!(ui, "Frametime     : {:?}", timestep.average());
        ui.separator();
        text_fmt!(ui, "Gpu info\n  Adapter:  {}\n  Version:  ", adapter_name);
    }
}

fn draw_ui_toggles(ctx: &mut InspectorContext) {
    let ui = ctx.ui;
    if ui.collapsing_header("Toggles", TreeNodeFlags::empty()) {
        let mut globals = ctx.resources.get_mut::<crate::Globals>().unwrap();

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

        tools::disabled(ui, || {
            let mut mode = false;
            ui.checkbox("Vsync", &mut mode);
        });

        ui.checkbox("Ibl enable", &mut globals.ibl_enable);
        ui.checkbox("Skybox enable", &mut globals.skybox_enable);
        ui.checkbox("Axis enable", &mut globals.axis_enable);
        ui.checkbox("BoundingBox", &mut globals.bbox_enable);
        ui.checkbox("Show demo window", &mut ctx.demo_open);

        if globals.ibl_enable {
            ui.slider("Exposure", 0.1, 8.0, &mut globals.exposure);

            let mut current_item = globals.tonemap_filter as usize;
            if ui.combo("Tonemap", &mut current_item, &TONEMAP_FILTERS, |item| {
                std::borrow::Cow::Borrowed(*item)
            }) {
                globals.tonemap_filter = current_item as u32;
            }
        } else {
            globals.tonemap_filter = 0;
            globals.exposure = 1.0;
        }
        if globals.skybox_enable {
            ui.separator();
            draw_ui_skybox_selector(ui, &ctx.resources);
        }
    }
}

fn draw_ui_camera(ctx: &InspectorContext) {
    let ui = ctx.ui;
    if ui.collapsing_header("Camera", TreeNodeFlags::empty()) {
        let mut camera = match ctx.resources.get_mut::<Camera>() {
            Some(camera) => camera,
            None => return,
        };

        ui.text(format!("Position: {:?}", camera.get_position()));
        ui.text(format!("FocalPoint: {:?}", camera.get_focal_point()));
        ui.text(format!(
            "Yaw/Pitch: {:.1} {:.1}",
            camera.get_yaw_pitch().0,
            camera.get_yaw_pitch().1
        ));
        if ui.button("Recenter From All") {
            camera.recenter_request = true;
        }
        if ui.button("Recenter From Selection") {}
        ui.separator();

        let mut fov = Deg::from(camera.fov).0;
        if Drag::new("Fov")
            .range(1.0f32, 179.0f32)
            .speed(1.0)
            .build(ui, &mut fov)
        {
            camera.fov = Rad(fov.to_radians());
        }

        let mut distance = camera.get_distance();
        if Drag::new("Distance")
            .range(0f32, 10f32)
            .speed(0.01)
            .build(ui, &mut distance)
        {
            camera.set_distance(distance);
        }

        let mut near = camera.near;
        let mut far = camera.far;
        if DragRange::new("Near/Far")
            .range(0.1, 100.0)
            .speed(0.01)
            .build(ui, &mut near, &mut far)
        {
            let near = near.max(0.1);
            let far = far.max(near + 0.1);
            camera.near = near;
            camera.far = far;
        }
    }
}

fn draw_ui_skybox_selector(ui: &Ui, resources: &Resources) {
    let registry = resources.get::<ImGuiTextureRegistry>().unwrap();
    let mut skybox_manager = resources.get_mut::<SkyboxManager>().unwrap();
    let device = resources.get::<wgpu::Device>().unwrap();
    let queue = resources.get::<wgpu::Queue>().unwrap();
    let mut texture_manager = resources.get_mut::<TextureManager>().unwrap();
    let gpu_resource_manager = resources
        .get::<std::sync::Arc<GPUResourceManager>>()
        .unwrap();

    let mut change_skybox = false;
    let hdr_path = skybox_manager.get_hdr_path();
    if let Some(id) = registry.ids.get(hdr_path) {
        let name = hdr_path
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
        let filepath = FileDialog::new().add_filter("hdr", &["hdr"]).pick_file();
        if let Some(filepath) = filepath {
            skybox_manager.change_skybox(
                &filepath,
                &device,
                &queue,
                &gpu_resource_manager,
                &mut texture_manager,
            );
        }
    }
}

