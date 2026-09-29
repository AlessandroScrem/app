use super::ui_layer::{Layer, UiContext};
use crate::editor::EntityCommand;
use crate::editor::{
    EntityId, InspectorData, InspectorSection, LightData, MaterialData, TransformData,
};
use imgui::*;

#[derive(Default)]
pub struct PropertyUi {
    inspector: Option<InspectorData>,
    draft: Option<InspectorData>,
    selection: Vec<EntityId>,
}

impl Layer for PropertyUi {
    fn update(&mut self, commands: &super::ui_commands::UiCommands) {
        let inspector = commands.inspector().cloned();

        if self.draft.as_ref().is_none_or(|draft| {
            inspector
                .as_ref()
                .is_none_or(|data| data.entity != draft.entity)
        }) {
            self.draft = inspector.clone();
        }

        self.inspector = inspector;
        self.selection = commands.selection().to_vec();
    }

    fn build(&mut self, ui: &Ui, ctx: &mut UiContext) {
        let window = ui
            .window("Properties")
            .size([420.0, 560.0], Condition::FirstUseEver);
        window.build(|| draw_inspector(ui, ctx, self.draft.as_mut(), &self.selection));
    }
}

fn draw_inspector(
    ui: &Ui,
    ctx: &mut UiContext,
    inspector: Option<&mut InspectorData>,
    selection: &[EntityId],
) {
    let Some(inspector) = inspector else {
        ui.text(format!("{} entities selected", selection.len()));
        for entity in selection {
            ui.text(format!("Entity: {}", entity));
        }
        return;
    };

    ui.text(format!("{}  [#{}]", inspector.name, inspector.entity));
    ui.separator();

    draw_inspector_name(ui, ctx, inspector);

    let entity = inspector.entity;
    for section in &mut inspector.sections {
        match section {
            InspectorSection::Transform(transform) => {
                draw_inspector_transform(ui, ctx, entity, transform);
            }
            InspectorSection::Mesh(mesh) => {
                if ui.collapsing_header("Mesh", TreeNodeFlags::DEFAULT_OPEN) {
                    ui.text(format!("Mesh: {}", mesh.id));
                }
            }
            InspectorSection::Materials(materials) => {
                draw_materials(ui, ctx, materials);
            }
            InspectorSection::BoundingBox(bbox) => {
                if ui.collapsing_header("Bounding Box", TreeNodeFlags::DEFAULT_OPEN) {
                    ui.text(format!("Local min: {:?}", bbox.min));
                    ui.text(format!("Local max: {:?}", bbox.max));
                    ui.separator();
                    ui.text(format!("Global min: {:?}", bbox.global_min));
                    ui.text(format!("Global max: {:?}", bbox.global_max));
                }
            }
            InspectorSection::Light(light) => {
                draw_light(ui, ctx, entity, light);
            }
        }
    }
}

fn draw_inspector_name(ui: &Ui, ctx: &mut UiContext, inspector: &mut InspectorData) {
    if !ui.collapsing_header(
        "Tag",
        TreeNodeFlags::DEFAULT_OPEN | TreeNodeFlags::ALLOW_ITEM_OVERLAP,
    ) {
        return;
    }

    let edited = ui.input_text("Name", &mut inspector.name).build();
    if edited {
        ctx.commands.send(EntityCommand::SetName {
            entity: inspector.entity,
            name: inspector.name.clone(),
        });
    }
}

fn draw_inspector_transform(
    ui: &Ui,
    ctx: &mut UiContext,
    entity: EntityId,
    transform: &mut TransformData,
) {
    if !ui.collapsing_header(
        "Transform",
        TreeNodeFlags::DEFAULT_OPEN | TreeNodeFlags::ALLOW_ITEM_OVERLAP,
    ) {
        return;
    }

    let (edited, activated, deactivated) = ui.group(|| {
        let translation_edited = Drag::new("Translation")
            .speed(0.1)
            .build_array(ui, &mut transform.translation);
        let translation_activated = ui.is_item_activated();
        let translation_deactivated = ui.is_item_deactivated_after_edit();

        let rotation_edited = Drag::new("Rotation")
            .speed(0.01)
            .build_array(ui, &mut transform.rotation);
        let rotation_activated = ui.is_item_activated();
        let rotation_deactivated = ui.is_item_deactivated_after_edit();

        let scale_edited = Drag::new("Scale")
            .speed(0.1)
            .build_array(ui, &mut transform.scale);
        let scale_activated = ui.is_item_activated();
        let scale_deactivated = ui.is_item_deactivated_after_edit();

        (
            translation_edited || rotation_edited || scale_edited,
            translation_activated || rotation_activated || scale_activated,
            translation_deactivated || rotation_deactivated || scale_deactivated,
        )
    });

    if activated {
        ctx.commands
            .send(EntityCommand::BeginTransformEdit { entity });
    }

    if edited {
        ctx.commands.send(EntityCommand::SetTransform {
            entity,
            transform: transform.clone(),
        });
    }

    if deactivated {
        ctx.commands
            .send(EntityCommand::EndTransformEdit { entity });
    }

    ui.separator();
    if ui.small_button("Reset Transform") {
        *transform = TransformData {
            translation: [0.0; 3],
            rotation: [0.0; 3],
            scale: [1.0; 3],
        };
        reset_transform(ctx, entity, transform.clone());
    }

    ui.same_line();
    if ui.small_button("Reset Position") {
        transform.translation = [0.0; 3];
        reset_transform(ctx, entity, transform.clone());
    }

    ui.same_line();
    if ui.small_button("Reset Rotation") {
        transform.rotation = [0.0; 3];
        reset_transform(ctx, entity, transform.clone());
    }
}

fn reset_transform(ctx: &mut UiContext, entity: EntityId, transform: TransformData) {
    ctx.commands
        .send(EntityCommand::BeginTransformEdit { entity });
    ctx.commands
        .send(EntityCommand::SetTransform { entity, transform });
    ctx.commands
        .send(EntityCommand::EndTransformEdit { entity });
}

fn draw_materials(ui: &Ui, ctx: &mut UiContext, materials: &mut [MaterialData]) {
    if !ui.collapsing_header("Materials", TreeNodeFlags::DEFAULT_OPEN) {
        return;
    }

    for material in materials {
        let id = ui.push_id(material.id.id.index.to_string());
        if ui.collapsing_header(&material.name, TreeNodeFlags::DEFAULT_OPEN) {
            draw_material(ui, ctx, material);
        }
        id.pop();
    }
}

fn draw_material(ui: &Ui, ctx: &mut UiContext, material: &mut MaterialData) {
    use crate::assets::material_desc::MaterialTextureSlot;

    let mut changed = false;

    for slot in MaterialTextureSlot::ALL {
        if !ui.collapsing_header(slot.as_str(), TreeNodeFlags::ALLOW_ITEM_OVERLAP) {
            continue;
        }

        if let Some(texture) = material.desc.texture(slot) {
            let mut enabled = material.desc.slot_get(slot);
            if ui.checkbox("Enabled", &mut enabled) {
                material.desc.slot_set(slot, enabled);
                changed = true;
            }

            if enabled {
                if let Some(texture_id) = ctx.textures.asset(texture) {
                    Image::new(texture_id, [96.0, 96.0]).build(ui);
                } else {
                    ui.text("Texture not available");
                }

                changed |= draw_texture_transform(ui, &mut material.desc, slot);
            }
        } else {
            ui.text("No texture");
        }

        changed |= draw_material_slot(ui, &mut material.desc, slot);
    }

    if changed {
        ctx.commands.send(crate::editor::AssetCommand::UpdateMaterial {
            id: material.id,
            desc: material.desc.clone(),
        });
    }
}

fn draw_material_slot(
    ui: &Ui,
    material: &mut crate::assets::material_desc::MaterialDesc,
    slot: crate::assets::material_desc::MaterialTextureSlot,
) -> bool {
    use crate::assets::material_desc::MaterialTextureSlot;

    match slot {
        MaterialTextureSlot::BaseColor => {
            let mut color: [f32; 4] = material.base_color_factor.into();
            let changed = ui
                .color_edit4_config("Factor", &mut color)
                .inputs(false)
                .build();
            if changed {
                material.base_color_factor = color.into();
            }
            changed
        }
        MaterialTextureSlot::Emissive => {
            let mut color: [f32; 4] = material.emissive_factor.into();
            let changed = ui
                .color_edit4_config("Factor", &mut color)
                .inputs(false)
                .build();
            if changed {
                material.emissive_factor = color.into();
            }
            changed
        }
        MaterialTextureSlot::Normal => Drag::new("Scale")
            .speed(0.01)
            .range(0.0, 1.0)
            .build(ui, &mut material.normal_scale),
        MaterialTextureSlot::MetallicRoughness => {
            let mut changed = false;
            changed |= Drag::new("Metallic")
                .speed(0.01)
                .range(0.0, 1.0)
                .build(ui, &mut material.metallic_factor);
            changed |= Drag::new("Roughness")
                .speed(0.01)
                .range(0.0, 1.0)
                .build(ui, &mut material.roughness_factor);
            changed
        }
        MaterialTextureSlot::Occlusion => Drag::new("Strength")
            .speed(0.01)
            .range(0.0, 1.0)
            .build(ui, &mut material.occlusion_strength),
        MaterialTextureSlot::Transmission => {
            let Some(transmission) = material.transmission.as_mut() else {
                return false;
            };
            let mut changed = Drag::new("Factor")
                .speed(0.01)
                .range(0.0, 1.0)
                .build(ui, &mut transmission.factor);
            changed |= Drag::new("IOR")
                .speed(0.01)
                .range(1.0, 2.5)
                .build(ui, &mut material.ior);
            changed
        }
        MaterialTextureSlot::Volume => {
            let Some(volume) = material.volume.as_mut() else {
                return false;
            };
            let mut changed = false;
            changed |= Drag::new("Thickness")
                .speed(0.01)
                .range(0.0, 1.0)
                .build(ui, &mut volume.thickness_factor);
            changed |= Drag::new("Distance")
                .speed(0.01)
                .range(0.0, 1.0)
                .build(ui, &mut volume.attenuation_distance);
            changed |= ui.color_edit3("Attenuation", &mut volume.attenuation_color);
            changed
        }
    }
}

fn draw_texture_transform(
    ui: &Ui,
    material: &mut crate::assets::material_desc::MaterialDesc,
    slot: crate::assets::material_desc::MaterialTextureSlot,
) -> bool {
    let Some(transform) = material.uvtransform_mut(slot) else {
        return false;
    };

    let mut changed = false;
    changed |= Drag::new("Offset")
        .speed(0.01)
        .build_array(ui, &mut transform.offset);
    changed |= Drag::new("Rotation")
        .speed(0.01)
        .build(ui, &mut transform.rotation);
    changed |= Drag::new("Scale")
        .speed(0.01)
        .build_array(ui, &mut transform.scale);
    changed
}

fn draw_light(ui: &Ui, ctx: &mut UiContext, entity: EntityId, light: &mut LightData) {
    if !ui.collapsing_header("Light", TreeNodeFlags::DEFAULT_OPEN) {
        return;
    }

    if draw_light_properties(ui, light) {
        ctx.commands.send(EntityCommand::SetLight {
            entity,
            light: light.clone(),
        });
    }

    if light.cast_shadow {
        if let Some(texture) = ctx.textures.shadow_map() {
            Image::new(texture, [200.0, 200.0]).build(ui);
        }
    }
}

fn draw_light_properties(ui: &Ui, light: &mut LightData) -> bool {
    let mut edited = false;
    edited |= ui.color_edit3("Color", &mut light.color);
    edited |= ui.checkbox("Directional", &mut light.directional);
    edited |= ui.checkbox("Cast Shadow", &mut light.cast_shadow);
    if light.cast_shadow {
        edited |= ui.checkbox("Frustum", &mut light.frustum);
    }
    edited
}
