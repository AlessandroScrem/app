use std::{
    collections::HashMap,
    path::PathBuf,
    time::{Duration, Instant},
};

use cgmath::{Deg, Rad};
use imgui::*;
use imgui_wgpu::{RawTextureConfig, Renderer, RendererConfig};
use imgui_winit_support::WinitPlatform;
use legion::{Entity, Resources, World};
use winit::window::Window;

use crate::{
    LightComponent, MeshComponent, TagComponent, TransformComponent,
    assets::texture_manager::TextureManager, camera::Camera,
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
            println!("add to registry {} with id {}", path.display(), id.id());
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

        resources.insert(renderer);
        resources.insert(registry);

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
            draw_window_general_info(ui, delta_s, &mut self.demo_open);
            draw_window_camera(ui, &resources);
            draw_window_entities(ui, world, &mut self.entity_selected);
            draw_window_properties(ui, world, &resources, self.entity_selected);

            draw_debug_window(ui, self.demo_open);
            draw_debug_texture(ui, &resources);
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

fn draw_window_general_info(ui: &imgui::Ui, delta_s: Duration, demo_open: &mut bool) {
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
            ui.checkbox("Show demo window", demo_open)
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
    let mut query = <(Entity, &TagComponent)>::query();

    let window = ui.window("Entities");
    window
        .size([300.0, 100.0], Condition::FirstUseEver)
        .position([0.0, 300.0], Condition::FirstUseEver)
        .build(|| {
            for (entity, tag) in query.iter(world) {
                if ui
                    .selectable_config(format!("{} {:?}", tag.name, entity))
                    .selected(selected.map(|e| e == *entity).unwrap_or(false))
                    .build()
                {
                    *selected = Some(*entity);
                }
            }
        });
}

fn draw_window_properties(
    ui: &imgui::Ui,
    world: &mut World,
    resources: &Resources,
    selected: Option<Entity>,
) {
    let entity = match selected {
        Some(entity) => entity,
        None => return,
    };

    let registry = resources.get::<ImGuiTextureRegistry>().unwrap();

    let window = ui.window(format!("Properties for {:?}", entity));
    window
        .size([300.0, 300.0], Condition::FirstUseEver)
        .position([0.0, 400.0], Condition::FirstUseEver)
        .build(|| {
            ui.separator();
            draw_ui_mesh(ui, world, &registry, entity.clone());
            ui.separator();
            draw_ui_light(ui, world, entity.clone());
        });
}

fn draw_ui_mesh(
    ui: &imgui::Ui,
    world: &mut World,
    registry: &ImGuiTextureRegistry,
    entity: Entity,
) {
    use legion::query::IntoQuery;
    let mut query = <(&mut MeshComponent, &mut TransformComponent)>::query();

    if let Ok((mesh, transform)) = query.get_mut(world, entity) {
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
        ui.collapsing_header("Light Properties", TreeNodeFlags::DEFAULT_OPEN);

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
