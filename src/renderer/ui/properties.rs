use super::*;
use std::path::PathBuf;

use imgui::{Drag, TreeNodeFlags};

use crate::{
    BoundingBoxComponent, DomainEvent, LightComponent, MeshComponent, TagComponent,
    TransformComponent, assets::material_manager::MaterialPBR,
    prelude::ui::registry::ImGuiTextureRegistry, text_fmt,
};

pub fn ui_properties(ui: &imgui::Ui, ctx: &mut UiContext) {
    ui.window("Properties")
        .size([300.0, 300.0], Condition::FirstUseEver)
        .build(|| {
            draw_entity_inspector(ui, ctx);
        });
}

pub fn draw_entity_inspector(ui: &imgui::Ui, ctx: &mut UiContext) {
    let Some(selected) = ctx.snapshot.selected else {
        return;
    };

    let cv = &mut ctx.snapshot.comp_view;

    if let Some(f) = &mut cv.tag {
        if f.draw_ui(ui) {
            ctx.commands
                .push_back(DomainEvent::UpdateTag(selected.clone(), f.clone()));
        }
    }

    if let Some(f) = &mut cv.transform {
        if f.draw_ui(ui) {
            ctx.commands
                .push_back(DomainEvent::UpdateTransform(selected.clone(), f.clone()));
        }
    }

    if let Some(f) = &mut cv.bounding_box {
        f.draw_ui(ui);
    }

    if let Some(f) = &mut cv.mesh {
        f.draw_ui(ui);
    }

    if let Some(f) = &mut cv.material {
        if f.draw_ui(ui, ctx.registry) {
            ctx.commands
                .push_back(DomainEvent::UpdateMaterial(selected.clone(), f.clone()));
        }
    }

    if let Some(f) = &mut cv.light {
        if f.draw_ui(ui) {
            ctx.commands
                .push_back(DomainEvent::UpdateLight(selected.clone(), f.clone()));
        }
    }
}

impl TagComponent {
    fn draw_ui(&mut self, ui: &Ui) -> bool {
        let tag = self;
        let mut dirty = false;
        if ui.collapsing_header(
            "TagComponent",
            TreeNodeFlags::DEFAULT_OPEN | TreeNodeFlags::ALLOW_ITEM_OVERLAP,
        ) {
            dirty |= ui.input_text("Name: ", &mut tag.name).build();
        }
        dirty
    }
}

impl MaterialPBR {
    fn draw_ui(&mut self, ui: &Ui, registry: &ImGuiTextureRegistry) -> bool {
        let material = self;
        let mut dirty = false;

        if ui.collapsing_header(
            "Material",
            TreeNodeFlags::DEFAULT_OPEN | TreeNodeFlags::ALLOW_ITEM_OVERLAP,
        ) {
            let main = &material.base_texture_path;
            let normal = &material.normal_texture_path;
            let roughness = &material.met_rough_texture_path;
            let emissive = &material.emissive_texture_path;
            let occlusion = &material.occlusion_texture_path;
            let name = format!("Material: {} ", material.name);

            if ui.collapsing_header(name, TreeNodeFlags::DEFAULT_OPEN | TreeNodeFlags::LEAF) {
                ui.text("Color");
                dirty |= ui.checkbox("Use##_ct", &mut material.use_color_texture);
                ui.same_line();
                draw_ui_texture_icon(ui, registry, main);
                ui.same_line();
                ui.disabled(material.use_color_texture, || {
                    let mut color: [f32; 4] = material.base_color_factor.into();
                    if ui.color_edit4("##Base Color", &mut color) {
                        material.base_color_factor = color.into();
                        dirty = true;
                    }
                });
                ui.separator();

                ui.text("Emissive");
                dirty |= ui.checkbox("Use##_em", &mut material.use_emissive_texture);
                ui.same_line();
                draw_ui_texture_icon(ui, registry, emissive);
                ui.same_line();
                ui.disabled(material.use_emissive_texture, || {
                    let mut color: [f32; 4] = material.emissive_factor.into();
                    if ui.color_edit4("##Emissive", &mut color) {
                        material.emissive_factor = color.into();
                        dirty = true;
                    }
                });
                ui.separator();

                ui.text("Occlusion");
                dirty |= ui.checkbox("Use##_occ", &mut material.use_occlusion_texture);
                ui.same_line();
                draw_ui_texture_icon(ui, registry, occlusion);
                ui.same_line();
                ui.disabled(material.use_occlusion_texture, || {
                    dirty |= Drag::new("##Occlusion")
                        .speed(0.01)
                        .range(0.0, 1.0)
                        .build(ui, &mut material.occlusion_strength);
                });
                ui.separator();

                ui.text("Metallic Roughness");
                dirty |= ui.checkbox("Use##_mr", &mut material.use_metal_roughness_texture);
                ui.same_line();
                draw_ui_texture_icon(ui, registry, roughness);
                ui.disabled(material.use_metal_roughness_texture, || {
                    dirty |= Drag::new("Met")
                        .speed(0.01)
                        .range(0.01, 1.0)
                        .build(ui, &mut material.metallic_factor);
                    dirty |= Drag::new("Rough")
                        .speed(0.01)
                        .range(0.01, 1.0)
                        .build(ui, &mut material.roughness_factor);
                });
                ui.separator();

                ui.text("Normal");
                dirty |= ui.checkbox("Use##_normal_texture", &mut material.use_normal_texture);
                ui.same_line();
                draw_ui_texture_icon(ui, registry, normal);
            }
        }
        dirty
    }
}

impl TransformComponent {
    fn draw_ui(&mut self, ui: &Ui) -> bool {
        let transform = self;
        let mut dirty = false;

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
                dirty = true;
            };
            if Drag::new("Rot[rad]").speed(0.01).build_array(ui, &mut rot) {
                transform.rotation = rot;
                dirty = true;
            };
            if Drag::new("Scale").speed(0.1).build_array(ui, &mut scale) {
                transform.scale = scale;
                dirty = true;
            };
            id.pop();
        }
        dirty
    }
}

impl BoundingBoxComponent {
    fn draw_ui(&self, ui: &Ui) -> bool {
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
        false
    }
}

impl MeshComponent {
    fn draw_ui(&mut self, ui: &Ui) -> bool {
        if ui.collapsing_header(
            "MeshComponent",
            TreeNodeFlags::DEFAULT_OPEN | TreeNodeFlags::ALLOW_ITEM_OVERLAP,
        ) {}
        false
    }
}

impl LightComponent {
    fn draw_ui(&mut self, ui: &Ui) -> bool {
        let mut dirty = false;

        let light = self;
        if ui.collapsing_header("Light Properties", TreeNodeFlags::DEFAULT_OPEN) {
            text_fmt!(ui, "Entity ID: {}", light.data.entity_id);
            let data = &mut light.data;
            dirty |= Drag::new("Position")
                .speed(0.1)
                .build_array(ui, &mut data.position);
            dirty |= ui.color_edit3("Color", &mut data.color);
            {
                let mut directional = data.directional != 0;
                if ui.checkbox("Directional", &mut directional) {
                    data.directional = directional as u32;
                    dirty = true;
                }
            }

            {
                let mut cast_shadow = data.cast_shadow != 0;
                if ui.checkbox("Cast Shadow", &mut cast_shadow) {
                    data.cast_shadow = cast_shadow as u32;
                    dirty = true;
                }
            }
        }
        dirty
    }
}

fn draw_ui_texture_icon(ui: &imgui::Ui, registry: &ImGuiTextureRegistry, name: &PathBuf) {
    if let Some(id) = registry.ids.get(name) {
        ui.image_button(name.to_str().unwrap(), *id, [25.0, 25.0]);
    }
}
