use std::{collections::HashMap, path::PathBuf};

use cgmath::{Deg, Rad};
use imgui::*;
use imgui_wgpu::{RawTextureConfig, Renderer, RendererConfig};
use imgui_winit_support::WinitPlatform;
use legion::{Entity, Resources, World};
use winit::window::Window;

use crate::{
    LightComponent, MeshComponent, TagComponent, TransformComponent,
    assets::texture_manager::TextureManager,
    camera::Camera,
    prelude::imgui_tools::ui_tools::Timestep,
    renderer::{
        gpu_manager::GPUResourceManager,
        gpu_renderer::{Hovered, PickPoint},
        uniform::ModelUniform,
    },
};

// registro imgui separato
pub struct ImGuiTextureRegistry {
    pub ids: HashMap<PathBuf, TextureId>,
}

impl ImGuiTextureRegistry {
    pub fn new() -> Self {
        Self {
            ids: HashMap::new(),
        }
    }
}

macro_rules! text_fmt {
($ui:expr, $($arg:tt)*) => {
    $ui.text(format!($($arg)*));
};
}
mod ui_tools {

    use std::time::{Duration, Instant};

    pub struct Timestep {
        time: Duration,     // durata del frame corrente (smoothed)
        last: Instant,      // istante dell'ultimo frame
        fps: f32,           // fps corrente (aggiornato ogni secondo)
        count: f32,         // conteggio frame nel secondo corrente
        timer: f32,         // timer accumulato per aggiornare fps
        avg_time: Duration, // media mobile del timestep
        avg_fps: f32,       // media mobile degli fps (stabile)
        samples: u32,       // numero di campioni accumulati
    }

    impl Timestep {
        pub fn new() -> Self {
            Self {
                time: Duration::from_secs_f32(0.0),
                last: Instant::now(),
                fps: 0.0,
                count: 0.0,
                timer: 0.0,
                avg_time: Duration::from_secs_f32(0.0),
                avg_fps: 0.0,
                samples: 0,
            }
        }

        /// Aggiorna il timestep in base al tempo trascorso dall'ultimo frame
        pub fn update(&mut self) {
            let now = Instant::now();
            let dt = now - self.last;
            self.last = now;

            // smoothing per rendere il valore più stabile
            const MEDIAN: f32 = 0.9;
            const VARIANT: f32 = 0.1;
            let blended = self.time.as_secs_f32() * MEDIAN + dt.as_secs_f32() * VARIANT;
            self.time = Duration::from_secs_f32(blended);

            // aggiorna media mobile del timestep (average in Duration)
            self.samples += 1;
            let avg_secs = (self.avg_time.as_secs_f32() * ((self.samples - 1) as f32)
                + dt.as_secs_f32())
                / (self.samples as f32);
            self.avg_time = Duration::from_secs_f32(avg_secs);

            // aggiorna FPS calcolato
            self.count += 1.0;
            self.timer += self.time.as_secs_f32();
            if self.timer >= 1.0 {
                self.fps = self.count;
                self.count = 0.0;
                self.timer = 0.0;

                // media mobile esponenziale per FPS (stabile e fluida)
                const FPS_SMOOTH: f32 = 0.9;
                self.avg_fps = self.avg_fps * FPS_SMOOTH + self.fps * (1.0 - FPS_SMOOTH);
            }
        }

        pub fn delta(&self) -> Duration {
            self.time
        }
        
        #[allow(dead_code)]
        pub fn fps(&self) -> u32 {
            self.fps as u32
        }

        /// Media del timestep dall'avvio (in Duration)
        pub fn average(&self) -> Duration {
            self.avg_time
        }

        /// Media mobile stabile degli FPS
        pub fn average_fps(&self) -> u32 {
            self.avg_fps as u32
        }
    }

    pub fn disabled<F>(ui: &imgui::Ui, func: F)
    where
        F: FnOnce(),
    {
        let _d = ui.begin_disabled(true);
        func();
    }

    pub fn set_dark_theme_colors(style: &mut imgui::Style) {
        const DARK_GREY: [f32; 4] = [0.1, 0.105, 0.11, 1.0];
        const COLD_GREY: [f32; 4] = [0.2, 0.205, 0.21, 1.0];
        const DARK_COLD_GREY: [f32; 4] = [0.15, 0.1505, 0.151, 1.0];
        const GREY: [f32; 4] = [0.28, 0.2805, 0.281, 1.0];
        const MEDIUM_GREY: [f32; 4] = [0.3, 0.305, 0.31, 1.0];
        const LIGHT_GREY: [f32; 4] = [0.38, 0.3805, 0.381, 1.0];

        // let DarkGrey: imgui::ImColor32 = imgui::ImColor32::from_rgb_f32s(0.1, 0.105, 0.11);

        let colors = &mut style.colors;

        colors[imgui::StyleColor::WindowBg as usize] = DARK_GREY;

        // Headers
        colors[imgui::StyleColor::Header as usize] = COLD_GREY;
        colors[imgui::StyleColor::HeaderHovered as usize] = MEDIUM_GREY;
        colors[imgui::StyleColor::HeaderActive as usize] = DARK_COLD_GREY;

        // Buttons
        colors[imgui::StyleColor::Button as usize] = COLD_GREY;
        colors[imgui::StyleColor::ButtonHovered as usize] = MEDIUM_GREY;
        colors[imgui::StyleColor::ButtonActive as usize] = DARK_COLD_GREY;

        // Frame BG
        colors[imgui::StyleColor::FrameBg as usize] = COLD_GREY;
        colors[imgui::StyleColor::FrameBgHovered as usize] = MEDIUM_GREY;
        colors[imgui::StyleColor::FrameBgActive as usize] = DARK_COLD_GREY;

        // Tabs
        colors[imgui::StyleColor::Tab as usize] = DARK_COLD_GREY;
        colors[imgui::StyleColor::TabHovered as usize] = LIGHT_GREY;
        colors[imgui::StyleColor::TabActive as usize] = GREY;
        colors[imgui::StyleColor::TabUnfocused as usize] = DARK_COLD_GREY;
        colors[imgui::StyleColor::TabUnfocusedActive as usize] = COLD_GREY;

        // Title
        colors[imgui::StyleColor::TitleBg as usize] = DARK_COLD_GREY;
        colors[imgui::StyleColor::TitleBgActive as usize] = DARK_COLD_GREY;
        colors[imgui::StyleColor::TitleBgCollapsed as usize] = DARK_COLD_GREY;
    }
}

// Sync texture with TextureManager textures
pub fn sync_with_registry(
    device: &wgpu::Device,
    manager: &TextureManager,
    registry: &mut ImGuiTextureRegistry,
    renderer: &mut imgui_wgpu::Renderer,
) {
    // record new textures
    for (path, tex) in &manager.textures {
        if !registry.ids.contains_key(path) {
            let texture_config = RawTextureConfig {
                label: None,
                sampler_desc: wgpu::SamplerDescriptor {
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Linear,
                    mipmap_filter: wgpu::FilterMode::Linear,
                    ..Default::default()
                },
            };
            let id = renderer
                .textures
                .insert(imgui_wgpu::Texture::from_raw_parts(
                    device,
                    renderer,
                    tex.inner.clone(),
                    tex.view.clone(),
                    None,
                    Some(&texture_config),
                    tex.extent,
                ));
            registry.ids.insert(path.clone(), id);
            // println!("add to registry {} with id {}", path.display(), id.id());
        }
    }

    // rimuove quelle che non esistono più nel texture manager
    registry.ids.retain(|path, id| {
        if !manager.textures.contains_key(path) {
            renderer.textures.remove(*id);
            println!(
                "remove from registry {} with id {}",
                path.display(),
                id.id()
            );
            false
        } else {
            true
        }
    });
}

pub struct ImguiState {
    pub context: imgui::Context,
    pub platform: WinitPlatform,
    pub clear_color: wgpu::Color,
    pub demo_open: bool,
    pub last_cursor: Option<MouseCursor>,
    pub entity_selected: Option<Entity>,
    ini_loaded: bool,
    timestep: Timestep,
}

impl ImguiState {
    pub fn create_imgui(window: &Window, resources: &mut legion::Resources) -> Self {
        let mut context = imgui::Context::create();

        let io = context.io_mut();
        io.config_flags.insert(ConfigFlags::DOCKING_ENABLE);
        io.config_flags.insert(ConfigFlags::VIEWPORTS_ENABLE);

        ui_tools::set_dark_theme_colors(context.style_mut());

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

        let last_cursor = None;
        let demo_open = false;

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

        let registry = ImGuiTextureRegistry::new();
        let timestep = Timestep::new();

        resources.insert(renderer);
        resources.insert(registry);

        Self {
            context,
            platform,
            clear_color,
            demo_open,
            last_cursor,
            entity_selected: None,
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

    pub fn update_ui(
        &mut self,
        window: &Window,
        world: &mut legion::World,
        resources: &mut legion::Resources,
    ) {
        self.timestep.update();
        let delta_s = self.timestep.delta();

        self.context.io_mut().update_delta_time(delta_s);

        self.platform
            .prepare_frame(self.context.io_mut(), &window)
            .expect("failed_to prepare frame");

        let ui = self.context.frame();
        {
            ui.dockspace_over_main_viewport();

            let mut win_pos = [0.0, 0.0];
            draw_window_settings(
                ui,
                &mut win_pos,
                &self.timestep,
                &mut self.demo_open,
                resources,
            );
            draw_window_entities(ui, &mut win_pos, world, &mut self.entity_selected);
            draw_window_properties(ui, &mut win_pos, world, &resources, self.entity_selected);

            draw_debug_window(ui, self.demo_open);
            draw_debug_texture(ui, &resources);
        }

        if self.last_cursor != ui.mouse_cursor() {
            self.last_cursor = ui.mouse_cursor();
            self.platform.prepare_render(ui, window);
        };

        self.load_ini_if_needed();

        let draw_data: &DrawData = self.context.render();
        let owned = OwnedDrawData::from(draw_data);
        resources.insert(owned);
    }
}

fn draw_window_settings(
    ui: &imgui::Ui,
    win_pos: &mut [f32; 2],
    timestep: &Timestep,
    demo_open: &mut bool,
    resources: &mut legion::Resources,
) {
    let mut globals = resources.get_mut::<crate::Globals>().unwrap();
    let hovered = resources.get::<Hovered>().unwrap();

    let tonemap_filters = [
        "ACES",
        "Filmic",
        "Lottes",
        "Reinhard",
        "Reinhard2",
        "Uchimura",
        "Uncharted2",
        "Exponential",
    ];

    let window = ui.window("Settings");
    let win_size = [300.0, 300.0];
    let adapter_name = resources.get::<wgpu::Adapter>().unwrap().get_info().name;
    window
        .size(win_size, Condition::FirstUseEver)
        .position(*win_pos, Condition::FirstUseEver)
        .build(|| {
            if ui.collapsing_header("Parameters", TreeNodeFlags::empty()) {
                let mouse_pos = ui.io().mouse_pos;
                ui.text(format!(
                    "Mouse position: ({:.1},{:.1})",
                    mouse_pos[0], mouse_pos[1]
                ));
                let point = resources.get::<PickPoint>().unwrap();
                text_fmt!(ui, "ResultGetPixel  : {} ", 0);
                text_fmt!(ui, "Mouse point  : {} {}  ", point.x, point.y);
                let hovered_entity_name = "Noname";
                let selected_entity_name = "Noname";
                let hovered_entity_id = hovered.0;
                let selected_entity_id = 0;
                text_fmt!(
                    ui,
                    "Hovered Entity  : {} ID: {}",
                    hovered_entity_name,
                    hovered_entity_id,
                );
                text_fmt!(
                    ui,
                    "Selected Entity : {} ID: {}",
                    selected_entity_name,
                    selected_entity_id,
                );
                ui.separator();
                ui_tools::disabled(ui, || {
                    text_fmt!(ui, "NumShaders         : {}", 0);
                    text_fmt!(ui, "NumTextures        : {}", 0);
                    text_fmt!(ui, "NumUniqueTextures  : {}", 0);
                    text_fmt!(ui, "Texture Memory Size: {}", 0);
                    text_fmt!(ui, "Memory Allocations : {}", 0);
                    text_fmt!(ui, "Memory Size        : {}", 0);
                });
            };
            if ui.collapsing_header("Statistics", TreeNodeFlags::DEFAULT_OPEN) {
                text_fmt!(ui, "FPS           : {:?}", timestep.average_fps());
                text_fmt!(ui, "Frametime     : {:?}", timestep.average());
                ui.separator();
                text_fmt!(ui, "Gpu info\n  Adapter:  {}\n  Version:  ", adapter_name);
            };

            if ui.collapsing_header("Toggles", TreeNodeFlags::empty()) {
                ui_tools::disabled(ui, || {
                    let mut mode = false;
                    ui.checkbox("Vsync", &mut mode);
                });

                ui.checkbox("Ibl enable", &mut globals.ibl_enable);
                ui.checkbox("Skybox enable", &mut globals.skybox_enable);
                ui.checkbox("Axis enable", &mut globals.axis_enable);
                ui.checkbox("BoundingBox", &mut globals.bbox_enable);
                ui.checkbox("Show demo window", demo_open);

                if globals.ibl_enable {
                    ui.slider("Exposure", 0.1, 8.0, &mut globals.exposure);

                    let mut current_item = globals.tonemap_filter as usize;
                    if ui.combo("Tonemap", &mut current_item, &tonemap_filters, |item| {
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
                    draw_ui_skybox_selector(ui, &resources);
                }
            }
            ui_tools::disabled(ui, || {
                if ui.collapsing_header("SSAO", TreeNodeFlags::empty()) {}
            });
            if ui.collapsing_header("Camera", TreeNodeFlags::empty()) {
                draw_ui_camera(ui, resources);
            }
        });
    win_pos[1] = win_pos[1] + win_size[1];
}

fn draw_ui_camera(ui: &imgui::Ui, resources: &Resources) {
    let mut camera = match resources.get_mut::<Camera>() {
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

fn draw_ui_skybox_selector(ui: &imgui::Ui, resources: &Resources) {
    let registry = resources.get::<ImGuiTextureRegistry>().unwrap();
    let mut skybox_manager = resources
        .get_mut::<super::skybox_manager::SkyboxManager>()
        .unwrap();
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
fn draw_window_entities(
    ui: &imgui::Ui,
    win_pos: &mut [f32; 2],
    world: &World,
    selected: &mut Option<Entity>,
) {
    use legion::query::IntoQuery;
    let mut query = <(Entity, &TagComponent)>::query();

    let win_size = [300.0, 100.0];
    let window = ui.window("Entities");
    window
        .size(win_size, Condition::FirstUseEver)
        .position(*win_pos, Condition::FirstUseEver)
        .build(|| {
            for (entity, tag) in query.iter(world) {
                // deselect when click on this window
                if ui.is_window_hovered() && ui.is_mouse_down(MouseButton::Left) {
                    *selected = None;
                }
                if ui
                    .selectable_config(format!("{} {:?}", tag.name, entity))
                    .selected(selected.map(|e| e == *entity).unwrap_or(false))
                    .build()
                {
                    *selected = Some(*entity);
                }
            }
        });
    win_pos[1] = win_pos[1] + win_size[1];
}

fn draw_window_properties(
    ui: &imgui::Ui,
    win_pos: &mut [f32; 2],
    world: &mut World,
    resources: &Resources,
    selected: Option<Entity>,
) {
    let entity = match selected {
        Some(entity) => entity,
        None => return,
    };

    let registry = resources.get::<ImGuiTextureRegistry>().unwrap();

    let win_size = [300.0, 300.0];
    let window = ui.window(format!("Properties for {:?}", entity));
    window
        .size(win_size, Condition::FirstUseEver)
        .position(*win_pos, Condition::FirstUseEver)
        .build(|| {
            ui.separator();
            draw_ui_mesh(ui, world, &registry, entity.clone());
            ui.separator();
            draw_ui_light(ui, world, entity.clone());
        });
    win_pos[1] = win_pos[1] + win_size[1];
}

fn draw_ui_mesh(
    ui: &imgui::Ui,
    world: &mut World,
    registry: &ImGuiTextureRegistry,
    entity: Entity,
) {
    use legion::query::IntoQuery;

    let mut query = <(&mut MeshComponent, &mut TransformComponent, &ModelUniform)>::query();
    if let Ok((mesh, transform, mu)) = query.get_mut(world, entity) {
        text_fmt!(ui, "Entity ID: {}", mu.entity_id);
        for submesh in mesh.data.submeshes.iter_mut() {
            let material = &mut submesh.material;
            let main = &material.main_texture;
            let normal = &material.normal_texture;
            let roughness = &material.metallic_roughness_texture;
            let mut color_use_texture = material.color_use_texture == 1;
            let mut metallic_use_texture = material.metallic_use_texture == 1;
            let mut roughness_use_texture = material.roughness_use_texture == 1;
            ui.checkbox("color override", &mut color_use_texture);
            ui.checkbox("metallic override", &mut metallic_use_texture);
            ui.checkbox("roughness override", &mut roughness_use_texture);
            material.color_use_texture = color_use_texture as u32;
            material.metallic_use_texture = metallic_use_texture as u32;
            material.roughness_use_texture = roughness_use_texture as u32;

            if !color_use_texture {
                let mut color: [f32; 4] = material.color.into();
                if ui.color_edit4("Color", &mut color) {
                    material.color = color.into();
                }
            };

            if !metallic_use_texture {
                Drag::new("Metallic")
                    .speed(0.01)
                    .range(0.01, 1.0)
                    .build(ui, &mut material.metallic);
            }
            if !roughness_use_texture {
                Drag::new("Roughness")
                    .speed(0.01)
                    .range(0.01, 1.0)
                    .build(ui, &mut material.roughness);
            }
            ui.separator();

            if let Some(id) = registry.ids.get(main) {
                let name = main
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("no name");
                ui.image_button(name, *id, [100.0, 100.0]);
                ui.same_line();
                ui.text(name);
                ui.separator();
            }
            if let Some(id) = registry.ids.get(normal) {
                let name = normal
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("no name");
                ui.image_button(name, *id, [100.0, 100.0]);
                ui.same_line();
                ui.text(name);
                ui.separator();
            }
            if let Some(id) = registry.ids.get(roughness) {
                let name = roughness
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("no name");
                ui.image_button(name, *id, [100.0, 100.0]);
                ui.same_line();
                ui.text(name);
                ui.separator();
            }
        }

        draw_ui_transform(ui, "Mesh Transform", transform);
        ui.separator();
    }
}

fn draw_ui_light(ui: &imgui::Ui, world: &mut World, entity: Entity) {
    use legion::query::IntoQuery;
    let mut query = <&mut LightComponent>::query();

    if let Ok(light) = query.get_mut(world, entity) {
        if ui.collapsing_header("Light Properties", TreeNodeFlags::DEFAULT_OPEN) {
            text_fmt!(ui, "Entity ID: {}", light.data.entity_id);
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
}

fn draw_ui_transform(ui: &imgui::Ui, name: &str, transform: &mut TransformComponent) {
    if ui.collapsing_header(
        name,
        TreeNodeFlags::DEFAULT_OPEN | TreeNodeFlags::ALLOW_ITEM_OVERLAP,
    ) {
        ui.text(format!("Position: {:?}", transform.position));
        ui.text(format!("Rotation[Deg]: {:?}", transform.rotation));
        ui.text(format!("Scale: {:?}", transform.scale));
        ui.separator();

        let id = ui.push_id(name);
        let mut pos = transform.position;
        let mut rot = transform.rotation;
        let mut scale = transform.scale;
        if Drag::new("Move").speed(0.1).build_array(ui, &mut pos) {
            transform.position = pos;
        };
        if Drag::new("Rot[rad]").speed(0.01).build_array(ui, &mut rot) {
            transform.rotation = rot;
        };
        if Drag::new("Scale").speed(0.1).build_array(ui, &mut scale) {
            transform.scale = scale;
        };
        id.pop();
    }
}

fn draw_debug_window(ui: &imgui::Ui, demo_open: bool) {
    if demo_open {
        ui.show_demo_window(&mut true);
    }
}

fn draw_debug_texture(ui: &imgui::Ui, resources: &legion::Resources) {
    let registry = resources.get::<ImGuiTextureRegistry>().unwrap();
    let debug_tex_path = std::path::Path::new("debug_texture");
    let name = debug_tex_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("no name");

    if let Some(id) = registry.ids.get(debug_tex_path) {
        let window = ui.window("Debug Texture");
        window
            .size([256.0, 256.0], Condition::FirstUseEver)
            .position([400.0, 0.0], Condition::FirstUseEver)
            .build(|| {
                ui.image_button(name, *id, [200.0, 200.0]);
                ui.same_line();
                ui.text(name);
                ui.separator();
            });
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
