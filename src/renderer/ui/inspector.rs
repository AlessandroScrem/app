use imgui::{Drag, TreeNodeFlags};
use legion::{Entity, Resources, World};

use crate::{
    BoundingBoxComponent, LightComponent, MeshComponent, TagComponent, TransformComponent,
    prelude::ui::registry::ImGuiTextureRegistry, text_fmt,
};

pub struct InspectorContext<'a> {
    pub ui: &'a imgui::Ui,
    pub resources: &'a mut Resources, // o ResourceManager
    pub selected: Option<Entity>,
    pub demo_open: &'a mut bool,
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

impl ComponentDrawer for BoundingBoxComponent {
    fn draw_component(&mut self, ctx: &mut InspectorContext) {
        let ui = ctx.ui;
        let bbox = &self.bounding_box;
        let gbbox = &self.global_bounding_box;
        if ui.collapsing_header(
            "BoundingBoxComponent",
            TreeNodeFlags::DEFAULT_OPEN | TreeNodeFlags::ALLOW_ITEM_OVERLAP,
        ) {
            ui.text(format!("Min: {:?}", bbox.min));
            ui.text(format!("Max: {:?}", bbox.max));
            ui.separator();
            ui.text(format!("GlobalMin: {:?}", gbbox.min));
            ui.text(format!("GLobalMax: {:?}", gbbox.max));
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
            ui.text(format!("Min: {:?}", mesh.data.vmin));
            ui.text(format!("Max: {:?}", mesh.data.vmax));
            ui.separator();
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

pub fn draw_entity_inspector(world: &mut World, ctx: &mut InspectorContext) {
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
        // BoundingBox
        if let Ok(comp) = entry.get_component_mut::<BoundingBoxComponent>() {
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

pub fn draw_ui_mesh_material(
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
