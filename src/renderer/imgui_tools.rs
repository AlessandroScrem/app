use std::time::{Duration, Instant};

use cgmath::{Deg, Rad};
use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::WinitPlatform;
use legion::{Entity, Resources, World};
use winit::window::Window;

use crate::{LightComponent, assets::mesh::Mesh, camera::Camera, transform::Transform};

pub struct ImguiState {
    pub context: imgui::Context,
    pub platform: WinitPlatform,
    pub clear_color: wgpu::Color,
    pub demo_open: bool,
    pub last_frame: Instant,
    pub last_cursor: Option<MouseCursor>,
    pub entity_selected: Option<Entity>,
}

impl ImguiState {
    pub fn create_imgui(window: &Window, resources: &mut legion::Resources) -> Self {
        let mut context = imgui::Context::create();
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

        let clear_color = wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };

        let last_frame = Instant::now();
        let last_cursor = None;
        let demo_open = true;

        let renderer = {
            let device = resources.get::<wgpu::Device>().unwrap();
            let queue = resources.get::<wgpu::Queue>().unwrap();
            let format = resources
                .get::<wgpu::SurfaceConfiguration>()
                .unwrap()
                .format;
            let renderer_config = RendererConfig {
                texture_format: format,
                ..Default::default()
            };
            Renderer::new(&mut context, &device, &queue, renderer_config)
        };

        resources.insert(renderer);

        Self {
            context,
            platform,
            clear_color,
            demo_open,
            last_frame,
            last_cursor,
            entity_selected: None,
        }
    }

    pub fn update_ui(
        &mut self,
        window: &Window,
        world: &mut legion::World,
        resources: &mut legion::Resources,
    ) {
        let delta_s = self.last_frame.elapsed();
        self.last_frame = Instant::now();

        self.context.io_mut().update_delta_time(delta_s);

        self.platform
            .prepare_frame(self.context.io_mut(), &window)
            .expect("failed_to prepare frame");

        let ui = self.context.frame();
        {
            draw_window_general_info(ui, delta_s);
            draw_window_camera(ui, &resources);

            draw_window_entities(ui, world, &mut self.entity_selected);
            draw_window_properties(ui, world, self.entity_selected);
            
        }

        if self.last_cursor != ui.mouse_cursor() {
            self.last_cursor = ui.mouse_cursor();
            self.platform.prepare_render(ui, window);
        };

        let draw_data: &DrawData = self.context.render();
        let owned = OwnedDrawData::from(draw_data);
        resources.insert(owned);
    }
}

fn draw_window_general_info(ui: &imgui::Ui, delta_s: Duration) {
    let window = ui.window("General info");
    window
        .size([300.0, 100.0], Condition::FirstUseEver)
        .position([0.0, 0.0], Condition::FirstUseEver)
        .build(|| {
            ui.separator();
            ui.text(format!("Frametime: {delta_s:?}"));
            let mouse_pos = ui.io().mouse_pos;
            ui.text(format!(
                "Mouse position: ({:.1},{:.1})",
                mouse_pos[0], mouse_pos[1]
            ));
        });
}

fn draw_window_camera(ui: &imgui::Ui, resources: &Resources) {
    let mut camera = match resources.get_mut::<Camera>() {
        Some(camera) => camera,
        None => return,
    };

    let window = ui.window("Camera");
    window
        .size([300.0, 200.0], Condition::FirstUseEver)
        .position([0.0, 100.0], Condition::FirstUseEver)
        .build(|| {
            ui.text(format!("Position: {:?}", camera.get_position()));
            ui.text(format!("FocalPoint: {:?}", camera.get_focal_point()));
            ui.text(format!(
                "Yaw/Pitch: {:.1} {:.1}",
                camera.get_yaw_pitch().0,
                camera.get_yaw_pitch().1
            ));
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
        });
}

fn draw_window_entities(ui: &imgui::Ui, world: &World, selected: &mut Option<Entity>) {
    use legion::query::IntoQuery;
    let mut query = <Entity>::query();

    let window = ui.window("Entities");
    window
        .size([300.0, 100.0], Condition::FirstUseEver)
        .position([0.0, 300.0], Condition::FirstUseEver)
        .build(|| {
            for entity in query.iter(world) {
                if ui
                    .selectable_config(format!("Entity {:?}", entity))
                    .selected(selected.map(|e| e == *entity).unwrap_or(false))
                    .build()
                {
                    *selected = Some(*entity);
                }
            }
        });
}

fn draw_window_properties(ui: &imgui::Ui, world: &mut World, selected: Option<Entity>) {
    let entity = match selected {
        Some(entity) => entity,
        None => return,
    };

    let window = ui.window(format!("Properties for {:?}", entity));
    window
        .size([300.0, 300.0], Condition::FirstUseEver)
        .position([0.0, 400.0], Condition::FirstUseEver)
        .build(|| {
            ui.separator();
            draw_ui_mesh(ui, world, entity.clone());
            ui.separator();
            draw_ui_light(ui, world, entity.clone());
        });
}

fn draw_ui_mesh(ui: &imgui::Ui, world: &mut World, entity: Entity) {
    use legion::query::IntoQuery;
    let mut query = <(&Mesh, &mut Transform)>::query();

    if let Ok((mesh, transform)) = query.get_mut(world, entity) {
        ui.collapsing_header(&mesh.name, TreeNodeFlags::DEFAULT_OPEN);
        draw_ui_transform(ui, "Mesh Transform", transform);
    }
}

fn draw_ui_light(ui: &imgui::Ui, world: &mut World, entity: Entity) {
    use legion::query::IntoQuery;
    let mut query = <&mut LightComponent>::query();

    if let Ok(light) = query.get_mut(world, entity) {
        ui.collapsing_header(&light.name, TreeNodeFlags::DEFAULT_OPEN);

        let data = &mut light.data;
        Drag::new("Position")
            .speed(0.1)
            .build_array(ui, &mut data.position);
        ui.color_edit3("Color", &mut data.color);
        {
            let mut directional = data.directional != 0;
            if ui.checkbox("Directional", &mut directional) {
                data.directional = directional as u32;
            }
        }

        {
            let mut cast_shadow = data.cast_shadow != 0;
            if ui.checkbox("Cast Shadow", &mut cast_shadow) {
                data.cast_shadow = cast_shadow as u32;
            }
        }
    }
}

fn draw_ui_transform(ui: &imgui::Ui, name: &str, transform: &mut Transform) {
    if ui.collapsing_header(
        name,
        TreeNodeFlags::DEFAULT_OPEN | TreeNodeFlags::ALLOW_ITEM_OVERLAP,
    ) {
        ui.text(format!("Position: {:?}", transform.position));
        ui.text(format!("Rotation[Deg]: {:?}", transform.rotation));
        ui.text(format!("Scale: {:?}", transform.scale));
        ui.separator();

        let id = ui.push_id(name);
        Drag::new("Move")
            .speed(0.1)
            .build_array(ui, &mut transform.position);
        Drag::new("Rot[rad]")
            .speed(0.01)
            .build_array(ui, &mut transform.rotation);
        Drag::new("Scale")
            .speed(0.1)
            .build_array(ui, &mut transform.scale);
        id.pop();
    }
}
