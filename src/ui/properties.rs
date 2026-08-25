use super::*;
use imgui::*;

use crate::app::app::SelectedEntity;
use crate::assets::MaterialId;
use crate::assets::material_desc::{MaterialDesc, MaterialTextureSlot};
use imgui::{Drag, TreeNodeFlags};

use crate::app::domain::events::{AssetEvent, DomainEvent, EntityEvent};
use crate::ecs::components::{
    BoundingBoxComponent, LightComponent, MeshComponent, TagComponent, TransformComponent,
};

pub struct PropertyUi {}

impl Layer for PropertyUi {
    fn build(&mut self, ui: &Ui, ctx: &mut UiContext) {
        ui.window("Properties")
            .size([300.0, 300.0], Condition::FirstUseEver)
            .build(|| {
                draw_entity_inspector(ui, ctx);
            });
    }
}

pub fn draw_entity_inspector(ui: &imgui::Ui, ctx: &mut UiContext) {
    let selected = match ctx.snapshot.selected {
        SelectedEntity::Single(entity) => entity,
        SelectedEntity::Multiple(_) | SelectedEntity::None => return,
    };
    
    let texture_resolver = ctx.snapshot.texture_resolver;
    let cv = &ctx.snapshot.comp_state;

    if let Some(f) = &mut cv.tag.clone() {
        if f.draw_ui(ui) {
            ctx.bus.send_domain(DomainEvent::Entity(EntityEvent::UpdateTag(
                selected.clone(),
                f.clone(),
            )));
        }
    }

    if let Some(f) = &mut cv.transform.clone() {
        if f.draw_ui(ui) {
            ctx.bus.send_domain(DomainEvent::Entity(EntityEvent::UpdateTransform(
                    selected.clone(),
                    f.clone(),
                )));
        }
    }

    if let Some(f) = &cv.bounding_box {
        f.draw_ui(ui);
    }

    if let Some(f) = &mut cv.mesh.clone() {
        f.draw_ui(ui);
    }

    if let Some(materials) = &cv.materials {
        if let Some((mat_updated, mat_id)) = draw_materials(ui, materials, texture_resolver) {
            trace!("Add AssetEvent::UpdateMaterial for id{}", mat_id);
            ctx.bus.send_domain(DomainEvent::Assets(AssetEvent::UpdateMaterial(
                    mat_id,
                    mat_updated,
                )));
        }
    }

    if let Some(f) = &mut cv.light.clone() {
        if f.draw_ui(ui, texture_resolver) {
            ctx.bus.send_domain(DomainEvent::Entity(EntityEvent::UpdateLight(
                selected.clone(),
                f.clone(),
            )));
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

use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static SELECTED_INDEX: RefCell<MaterialId> = RefCell::new(MaterialId::default());
}

impl MaterialDesc {
    fn draw_ui_slot(
        &mut self,
        ui: &imgui::Ui,
        slot: MaterialTextureSlot,
        resolver: &dyn UiTextureResolver,
    ) -> bool {
        let mut dirty = false;
        let label = slot.as_str();
        let iconsize = [ui.text_line_height(), ui.text_line_height()];
        let material = self;

        if let Some(id) = material.texture(slot) {
            ui.text(label);

            let mut use_texture = material.slot_get(slot);

            if ui.checkbox(&format!("Use##{}", label), &mut use_texture) {
                material.slot_set(slot, use_texture);
                dirty = true;
            }

            if use_texture {
                ui.same_line();
                draw_ui_texture_icon(ui, resolver.resolve(UiTexture::Engine(id)), iconsize);
                dirty |= draw_texture_transform(ui, material, slot);
            } else {
                dirty |= slot.draw_ui(ui, material);
            }
        } else {
            ui.disabled(true, || {
                ui.text(label);
            });
            dirty |= slot.draw_ui(ui, material);
        }
        dirty
    }
}

impl MaterialTextureSlot {
    fn draw_ui(self, ui: &Ui, material: &mut MaterialDesc) -> bool {
        match self {
            MaterialTextureSlot::BaseColor => {
                let mut color: [f32; 4] = material.base_color_factor.into();
                ui.same_line();

                let changed = ui
                    .color_edit4_config("##BaseColor", &mut color)
                    .inputs(false)
                    .build();

                if changed {
                    material.base_color_factor = color.into();
                }

                changed
            }
            MaterialTextureSlot::Emissive => {
                let mut color: [f32; 4] = material.emissive_factor.into();
                ui.same_line();

                let changed = ui
                    .color_edit4_config("##Emissive", &mut color)
                    .inputs(false)
                    .build();

                if changed {
                    material.emissive_factor = color.into();
                }

                changed
            }
            MaterialTextureSlot::Normal => {
                let changed = Drag::new("##Normal")
                    .speed(0.01)
                    .range(0.0, 1.0)
                    .build(ui, &mut material.normal_scale);
                changed
            }
            MaterialTextureSlot::MetallicRoughness => {
                let mut changed = false;

                changed |= Drag::new("MetFactor")
                    .speed(0.01)
                    .range(0.01, 1.0)
                    .build(ui, &mut material.metallic_factor);

                changed |= Drag::new("RoughFactor")
                    .speed(0.01)
                    .range(0.01, 1.0)
                    .build(ui, &mut material.roughness_factor);

                changed
            }
            MaterialTextureSlot::Occlusion => {
                let changed = Drag::new("##Occlusion")
                    .speed(0.01)
                    .range(0.0, 1.0)
                    .build(ui, &mut material.occlusion_strength);
                changed
            }
            MaterialTextureSlot::Transmission => {
                if let Some(transmission) = material.transmission.as_mut() {
                    let mut changed = false;
                    changed |= Drag::new("Transmission")
                        .speed(0.01)
                        .range(0.0, 1.0)
                        .build(ui, &mut transmission.factor);

                    changed |= Drag::new("Ior")
                        .speed(0.01)
                        .range(1.0, 2.5)
                        .build(ui, &mut material.ior);
                    changed
                } else {
                    false
                }
            }
            MaterialTextureSlot::Volume => {
                if let Some(volume) = material.volume.as_mut() {
                    let mut changed = false;
                    ui.same_line();
                    changed |= ui
                        .color_edit3_config("##AttColor", &mut volume.attenuation_color)
                        .inputs(false)
                        .build();
                    changed |= Drag::new("Tick-Factor")
                        .speed(0.01)
                        .range(0.0, 1.0)
                        .build(ui, &mut volume.thickness_factor);
                    changed |= Drag::new("Att-Dist")
                        .speed(0.01)
                        .range(0.0, 1.0)
                        .build(ui, &mut volume.attenuation_distance);
                    changed
                } else {
                    false
                }
            }
        }
    }
}

fn draw_sheen_ui(ui: &Ui, material: &mut MaterialDesc) -> bool {
    if let Some(sheen) = material.sheen.as_mut() {
        let mut changed = false;

        ui.text("Sheen");
        ui.same_line();
        changed |= ui
            .color_edit3_config("##SheenColor", &mut sheen.color_factor)
            .inputs(false)
            .build();

        changed |= Drag::new("SheenRoughness")
            .speed(0.01)
            .range(0.01, 1.0)
            .build(ui, &mut sheen.roughness_factor);

        changed
    } else {
        false
    }
}

fn draw_materials(
    ui: &Ui,
    materials: &HashMap<MaterialId, MaterialDesc>,
    resolver: &dyn UiTextureResolver,
) -> Option<(MaterialDesc, MaterialId)> {
    let mut dirty = false;
    let mut result: Option<(MaterialDesc, MaterialId)> = None;

    if ui.collapsing_header(
        "Material",
        TreeNodeFlags::DEFAULT_OPEN | TreeNodeFlags::ALLOW_ITEM_OVERLAP,
    ) {
        if materials.is_empty() {
            trace!("materials empty");
            return None;
        }

        if let Some(_t) = ui.begin_table("", 2) {
            // -- Left column: Material list  ---
            ui.table_next_column();

            SELECTED_INDEX.with(|id_cell| {
                let mut selected_id = *id_cell.borrow();

                let mut keys: Vec<MaterialId> = materials.keys().cloned().collect();
                keys.sort_by_key(|id| id.id.index);

                // select first id if not exist
                if !materials.contains_key(&selected_id) {
                    selected_id = keys[0];
                }

                let listbox_height = materials.len() as f32 * ui.text_line_height_with_spacing();
                ui.child_window("##Material list")
                    .size([0.0, listbox_height])
                    .build(|| {
                        for id in keys {
                            let label = id.to_string();
                            let is_selected = id == selected_id;

                            // highlight selection if selected
                            if ui.selectable_config(label).selected(is_selected).build() {
                                selected_id = id;
                            }

                            // initial focus
                            if is_selected {
                                ui.set_item_default_focus();
                            }
                        }
                    });

                *id_cell.borrow_mut() = selected_id;
            });

            // --- Right column: material controls ---
            ui.table_next_column();
            SELECTED_INDEX.with(|idx_cell| {
                let selected_id = *idx_cell.borrow();

                use crate::assets::material_desc::MaterialTextureSlot::*;

                if let Some(mut material) = materials.get(&selected_id).cloned() {
                    let name = material.get_name();

                    if ui.collapsing_header(name, TreeNodeFlags::DEFAULT_OPEN | TreeNodeFlags::LEAF)
                    {
                        dirty |= material.draw_ui_slot(ui, BaseColor, resolver);
                        ui.separator();
                        dirty |= material.draw_ui_slot(ui, Emissive, resolver);
                        ui.separator();
                        dirty |= material.draw_ui_slot(ui, Occlusion, resolver);
                        ui.separator();
                        dirty |= material.draw_ui_slot(ui, MetallicRoughness, resolver);
                        ui.separator();
                        dirty |= material.draw_ui_slot(ui, Normal, resolver);
                        ui.separator();
                        dirty |= material.draw_ui_slot(ui, Transmission, resolver);
                        ui.separator();
                        dirty |= material.draw_ui_slot(ui, Volume, resolver);
                        ui.separator();
                        dirty |= draw_sheen_ui(ui, &mut material);
                    }

                    if dirty {
                        result = Some((material.clone(), selected_id.clone()));
                    }
                }
            });
        } // columns
    } // collapsing header
    result
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

        if ui.collapsing_header("BoundingBoxComponent", TreeNodeFlags::ALLOW_ITEM_OVERLAP) {
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
        if ui.collapsing_header("MeshComponent", TreeNodeFlags::ALLOW_ITEM_OVERLAP) {}
        false
    }
}

impl LightComponent {
    fn draw_ui(&mut self, ui: &Ui, resolver: &dyn UiTextureResolver) -> bool {
        let mut dirty = false;

        let light = self;
        if ui.collapsing_header("Light Properties", TreeNodeFlags::DEFAULT_OPEN) {
            let mut position = light.get_position();
            if Drag::new("Position")
                .speed(0.1)
                .build_array(ui, &mut position)
            {
                dirty = true;
                light.update_position(position);
            };

            dirty |= ui.color_edit3("Color", &mut light.color);
            {
                let mut enabled = light.enabled;
                if ui.checkbox("Enabled", &mut enabled) {
                    light.enabled = enabled;
                    dirty = true;
                }
            }
            {
                let mut directional = light.directional;
                if ui.checkbox("Directional", &mut directional) {
                    light.directional = directional;
                    dirty = true;
                }
            }

            {
                let mut cast_shadow = light.cast_shadow;
                if ui.checkbox("Cast Shadow", &mut cast_shadow) {
                    light.cast_shadow = cast_shadow;
                    dirty = true;
                }
            }
            light.cast_shadow.then(|| {
                let mut frustum = light.frustum;
                ui.same_line();
                if ui.checkbox("Frustum", &mut frustum) {
                    light.frustum = frustum;
                    dirty = true;
                }
            });
        }
        if light.enabled & light.cast_shadow {
            let iconsize = [200.0, 200.0];
            draw_ui_texture_icon(ui, resolver.resolve(UiTexture::ShadowMap), iconsize);
        }

        dirty
    }
}

fn draw_ui_texture_icon(ui: &imgui::Ui, id: Option<TextureId>, size: [f32; 2]) {
    if let Some(id) = id {
        ui.image_button("no name", id, size);
    }
}

fn draw_texture_transform(
    ui: &imgui::Ui,
    material: &mut MaterialDesc,
    slot: MaterialTextureSlot,
) -> bool {
    let mut dirty = false;

    if let Some(transform) = material.uvtransform_mut(slot) {
        let id = ui.push_id(slot.as_str());

        let offset = &mut transform.offset;
        let rotation = &mut transform.rotation;
        let scale = &mut transform.scale;

        dirty |= Drag::new("Offset")
            .range(-1.0, 1.0)
            .speed(0.01)
            .build_array(ui, offset);
        dirty |= Drag::new("Rotation").speed(0.01).build(ui, rotation);
        dirty |= Drag::new("Scale").speed(0.5).build_array(ui, scale);

        id.pop();
    }

    return dirty;
}
