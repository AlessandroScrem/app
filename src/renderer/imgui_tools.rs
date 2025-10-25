use std::{collections::HashMap, path::PathBuf};

use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::WinitPlatform;
use legion::*;
use winit::window::Window;

use crate::{
    HierarchyComponent, LightComponent, MeshComponent, TagComponent, TransformComponent,
    assets::texture_manager::TextureManager,
    camera::Camera,
    math::{Deg, Rad},
    picking::PickObject,
    renderer::gpu_manager::GPUResourceManager,
    timestep::Timestep,
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

pub struct InspectorContext<'a> {
    pub ui: &'a imgui::Ui,
    pub resources: &'a mut Resources, // o ResourceManager
    pub selected: Option<Entity>,
    pub demo_open: & 'a mut bool,
}

trait ComponentDrawer {
    fn draw_component(&mut self, ctx: &mut InspectorContext);
}

impl ComponentDrawer for TagComponent {
    fn draw_component(&mut self, ctx: &mut InspectorContext) {
        let ui = ctx.ui;
        let tag = self;
        if ui.collapsing_header(
            "TagComponent",
            TreeNodeFlags::DEFAULT_OPEN | TreeNodeFlags::ALLOW_ITEM_OVERLAP,
        ) {
            ui.text(format!("Name: {}", tag.name));
        }
    }
}

impl ComponentDrawer for TransformComponent {
    fn draw_component(&mut self, ctx: &mut InspectorContext) {
        let ui = ctx.ui;
        let transform = self;
        if ui.collapsing_header(
            "TransformComponent",
            TreeNodeFlags::DEFAULT_OPEN | TreeNodeFlags::ALLOW_ITEM_OVERLAP,
        ) {
            ui.text(format!("Position: {:?}", transform.position));
            ui.text(format!("Rotation[Deg]: {:?}", transform.rotation));
            ui.text(format!("Scale: {:?}", transform.scale));
            ui.separator();

            let id = ui.push_id_ptr(transform);
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
}

impl ComponentDrawer for MeshComponent {
    fn draw_component(&mut self, ctx: &mut InspectorContext) {
        let ui = ctx.ui;
        let mesh = self;
        let registry = ctx.resources.get::<ImGuiTextureRegistry>().unwrap();
        if ui.collapsing_header(
            "MeshComponent",
            TreeNodeFlags::DEFAULT_OPEN | TreeNodeFlags::ALLOW_ITEM_OVERLAP,
        ) {
            ui.text(format!("Mesh: {}", mesh.data.name));
            for (id, submesh) in mesh.data.submeshes.iter_mut().enumerate() {
                ui.text(format!("Material id {}", id));
                draw_ui_mesh_material(ui, &registry, &mut submesh.material);
            }
        }
    }
}

impl ComponentDrawer for LightComponent {
    fn draw_component(&mut self, ctx: &mut InspectorContext) {
        let ui = ctx.ui;
        let light = self;
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

fn draw_entity_inspector(world: &mut World, ctx: &mut InspectorContext) {
    if let Some(entity) = ctx.selected {
        let mut entry = world.entry(entity).unwrap();

        // Tag
        if let Ok(comp) = entry.get_component_mut::<TagComponent>() {
            comp.draw_component(ctx);
        }
        // Transform
        if let Ok(comp) = entry.get_component_mut::<TransformComponent>() {
            comp.draw_component(ctx);
        }
        // Mesh
        if let Ok(comp) = entry.get_component_mut::<MeshComponent>() {
            comp.draw_component(ctx);
        }
        // Light
        if let Ok(comp) = entry.get_component_mut::<LightComponent>() {
            comp.draw_component(ctx);
        }
    }
    // Qui puoi aggiungere altri componenti
}

pub struct ImguiState {
    pub context: imgui::Context,
    pub platform: WinitPlatform,
    pub clear_color: wgpu::Color,
    pub demo_open: bool,
    pub last_cursor: Option<MouseCursor>,
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
            demo_open: false,
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

        let selected = resources.get::<PickObject>().unwrap().selected;

        let ui = self.context.frame();
        {
            let mut ctx = InspectorContext {
                ui,
                resources,
                selected,
                demo_open: &mut self.demo_open,
            };
            ui.dockspace_over_main_viewport();

            let mut win_pos = [0.0, 0.0];
            draw_window_settings(&mut win_pos, &self.timestep, &mut ctx);
            draw_window_entities(&mut win_pos, world, &mut ctx);
            draw_window_properties(&mut win_pos, world, &mut ctx);

            draw_debug_window(&ctx);
            draw_debug_texture(&ctx);
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
    win_pos: &mut [f32; 2],
    timestep: &Timestep,
    ctx: &mut InspectorContext,
) {
    let resources = &ctx.resources;
    let ui = ctx.ui;
    let mut globals = resources.get_mut::<crate::Globals>().unwrap();
    let pick_object = resources.get::<PickObject>().unwrap();

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
    // let selected: Entity = crate::entities::EntityRawU64::from_raw_u64(selected.0);

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
                ui.checkbox("Show demo window", &mut ctx.demo_open);

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

fn draw_window_entities(win_pos: &mut [f32; 2], world: &mut World, ctx: &mut InspectorContext) {
    let resources = &ctx.resources;
    let ui = ctx.ui;
    use legion::query::IntoQuery;
    let mut pick_object = resources.get_mut::<PickObject>().unwrap();
    let mut hierarchy_query = <(Entity, &HierarchyComponent)>::query();
    let mut no_hierarchy_query =
        <(Entity, &TagComponent)>::query().filter(!component::<HierarchyComponent>());

    fn handle_deselection(ui: &imgui::Ui, pick_object: &mut PickObject) {
        // Deseleziona solo se clicchi nella finestra stessa
        // ma non sopra un widget/interazione
        if ui.is_window_hovered()
            && ui.is_mouse_clicked(MouseButton::Left)
            && !ui.is_any_item_hovered()
        {
            pick_object.select(None);
        }
    }

    let win_size = [300.0, 100.0];
    let window = ui.window("Entities");
    window
        .size(win_size, Condition::FirstUseEver)
        .position(*win_pos, Condition::FirstUseEver)
        .build(|| {
            // deselect if clicked on empty
            handle_deselection(ui, &mut pick_object);
            // draw hierarchy components

            ui.group(|| {
                for (entity, hirarchy) in hierarchy_query.iter(world) {
                    if hirarchy.parent.is_none() {
                        draw_entity_node_recurse(ui, entity.clone(), world, &mut pick_object);
                    }
                }
            });

            if let Some(selected) = pick_object.selected {
                // add Context menu to group
                if hierarchy_query
                    .get(world, selected)
                    .is_ok_and(|e| e.1.parent == None)
                {
                    if ui.is_item_hovered() && ui.is_mouse_clicked(imgui::MouseButton::Right) {
                        ui.open_popup("entity_context");
                    }
                    if let Some(popup) = ui.begin_popup("entity_context") {
                        if ui.menu_item("Add Parent to Node") {
                            crate::entities::add_parent(selected.clone(), world);
                        }
                        popup.end();
                    }
                }
            }

            ui.separator();

            for (entity, tag) in no_hierarchy_query.iter(world) {
                if ui
                    .selectable_config(format!("{} {:?}", tag.name, entity))
                    .selected(pick_object.selected.map(|e| e == *entity).unwrap_or(false))
                    .build()
                {
                    pick_object.select(Some(*entity));
                    println!("Selected");
                }
            }
        });

    win_pos[1] = win_pos[1] + win_size[1];
}

fn draw_entity_node_recurse(ui: &Ui, entity: Entity, world: &World, pick_object: &mut PickObject) {
    let (name, children) = {
        let entry = world.entry_ref(entity).unwrap();
        let tag = entry.get_component::<TagComponent>().unwrap();
        let hierarchy = entry.get_component::<HierarchyComponent>().unwrap();
        (tag.name.clone(), hierarchy.children.clone())
    };

    let is_selected = pick_object.selected.is_some_and(|e| e == entity);
    let flags = TreeNodeFlags::SPAN_AVAIL_WIDTH;
    let flags = if children.is_empty() {
        flags | TreeNodeFlags::LEAF
    } else {
        flags
    };
    let flags = if is_selected {
        flags | TreeNodeFlags::SELECTED
    } else {
        flags
    };

    ui.tree_node_config(name.clone())
        .flags(flags)
        .default_open(true)
        .build(|| {
            // Controlla se il nodo è stato cliccato e aggiorna la selezione
            if ui.is_item_clicked() {
                pick_object.select(Some(entity));
            }
            for child in children {
                draw_entity_node_recurse(ui, child, world, pick_object);
            }
        });
}

fn draw_window_properties(win_pos: &mut [f32; 2], world: &mut World, ctx: &mut InspectorContext) {
    if let Some(entity) = ctx.selected.as_ref() {
        let win_size = [300.0, 300.0];
        let window = ctx.ui.window(format!("Properties for {:?}", entity));
        window
            .size(win_size, Condition::FirstUseEver)
            .position(*win_pos, Condition::FirstUseEver)
            .build(|| {
                draw_entity_inspector(world, ctx);
            });
        win_pos[1] = win_pos[1] + win_size[1];
    };
}

fn draw_ui_mesh_material(
    ui: &imgui::Ui,
    registry: &ImGuiTextureRegistry,
    material: &mut crate::assets::material_manager::Material,
) {
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

fn draw_debug_window(ctx: &InspectorContext) {
    if *ctx.demo_open {
        ctx.ui.show_demo_window(&mut true);
    }
}

fn draw_debug_texture(ctx: &InspectorContext) {
    let registry = ctx.resources.get::<ImGuiTextureRegistry>().unwrap();
    let ui = ctx.ui;

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
