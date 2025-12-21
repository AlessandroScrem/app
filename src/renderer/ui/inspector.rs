use std::path::PathBuf;

use imgui::{Drag, TreeNodeFlags};
use legion::{Entity, Resources, World};

use crate::{
    BoundingBoxComponent, LightComponent, MeshComponent, TagComponent, TransformComponent,
    assets::material_manager::MaterialManager, prelude::ui::registry::ImGuiTextureRegistry,
    text_fmt,
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
            ui.text("Local");
            ui.text(format!("Min: {:?}", bbox.min));
            ui.text(format!("Max: {:?}", bbox.max));
            ui.separator();
            ui.text("Global");
            ui.text(format!("Min: {:?}", gbbox.min));
            ui.text(format!("Max: {:?}", gbbox.max));
        }
    }
}

impl ComponentDrawer for MeshComponent {
    fn draw_component(&mut self, ctx: &mut InspectorContext) {
        let ui = ctx.ui;
        let mesh = self;
        let registry = ctx.resources.get::<ImGuiTextureRegistry>().unwrap();
        let mut material_manager = ctx.resources.get_mut::<MaterialManager>().unwrap();
        if ui.collapsing_header(
            "MeshComponent",
            TreeNodeFlags::DEFAULT_OPEN | TreeNodeFlags::ALLOW_ITEM_OVERLAP,
        ) {
            for submesh in mesh.data.submeshes.iter_mut() {
                let material = material_manager.get_mut(&submesh.material);
                draw_ui_mesh_material(ui, &registry, &mut material.material_pbr);
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

fn draw_ui_texture_icon(ui: &imgui::Ui, registry: &ImGuiTextureRegistry, name: &PathBuf) {
    if let Some(id) = registry.ids.get(name) {
        ui.image_button(name.to_str().unwrap(), *id, [25.0, 25.0]);
    }
}

fn draw_ui_mesh_material(
    ui: &imgui::Ui,
    registry: &ImGuiTextureRegistry,
    material: &mut crate::assets::material_manager::MaterialPBR,
) {
    let main = &material.base_texture_path;
    let normal = &material.normal_texture_path;
    let roughness = &material.met_rough_texture_path;
    let emissive = &material.emissive_texture_path;
    let occlusion = &material.occlusion_texture_path;
    let name = format!("Material: {} ", material.name);

    if ui.collapsing_header(name, TreeNodeFlags::DEFAULT_OPEN | TreeNodeFlags::LEAF) {
        ui.text("Color");
        ui.checkbox("Use##_ct", &mut material.use_color_texture);
        ui.same_line();
        draw_ui_texture_icon(ui, registry, main);
        ui.same_line();
        ui.disabled(material.use_color_texture, || {
            let mut color: [f32; 4] = material.base_color_factor.into();
            if ui.color_edit4("##Base Color", &mut color) {
                material.base_color_factor = color.into();
            }
        });
        ui.separator();

        ui.text("Emissive");
        ui.checkbox("Use##_em", &mut material.use_emissive_texture);
        ui.same_line();
        draw_ui_texture_icon(ui, registry, emissive);
        ui.same_line();
        ui.disabled(material.use_emissive_texture, || {
            let mut color: [f32; 4] = material.emissive_factor.into();
            if ui.color_edit4("##Emissive", &mut color) {
                material.emissive_factor = color.into();
            }
        });
        ui.separator();

        ui.text("Occlusion");
        ui.checkbox("Use##_occ", &mut material.use_occlusion_texture);
        ui.same_line();
        draw_ui_texture_icon(ui, registry, occlusion);
        ui.same_line();
        ui.disabled(material.use_occlusion_texture, || {
            Drag::new("##Occlusion")
                .speed(0.01)
                .range(0.0, 1.0)
                .build(ui, &mut material.occlusion_strength);
        });
        ui.separator();

        ui.text("Metallic Roughness");
        ui.checkbox("Use##_mr", &mut material.use_metal_roughness_texture);
        ui.same_line();
        draw_ui_texture_icon(ui, registry, roughness);
        ui.disabled(material.use_metal_roughness_texture, || {
            Drag::new("Met")
                .speed(0.01)
                .range(0.01, 1.0)
                .build(ui, &mut material.metallic_factor);
            Drag::new("Rough")
                .speed(0.01)
                .range(0.01, 1.0)
                .build(ui, &mut material.roughness_factor);
        });
        ui.separator();

        ui.text("Normal");
        ui.checkbox("Use##_normal_texture", &mut material.use_normal_texture);
        ui.same_line();
        draw_ui_texture_icon(ui, registry, normal);
    }
}
