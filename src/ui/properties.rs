use super::ui_commands::UiCommands;
use super::ui_layer::{Layer, UiContext};
use crate::assets::material_desc::{MaterialDesc, MaterialTextureSlot};
use crate::editor::{
    AssetCommand, EntityCommand, EntityId, InspectorData, InspectorSection, LightData,
    MaterialData, TransformData,
};
use imgui::{Condition, Drag, Image, TreeNodeFlags, Ui};

#[derive(Default)]
pub struct PropertyUi {
    inspector: Option<InspectorData>,
    draft: Option<InspectorData>,
    selection: Vec<EntityId>,
}

impl Layer for PropertyUi {
    fn update(&mut self, commands: &UiCommands) {
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
            .size([620.0, 560.0], Condition::FirstUseEver);
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
            InspectorSection::Mesh(mesh) => draw_mesh(ui, mesh),
            InspectorSection::Materials(materials) => draw_materials(ui, ctx, materials),
            InspectorSection::BoundingBox(bbox) => draw_bounding_box(ui, bbox),
            InspectorSection::Light(light) => draw_light(ui, ctx, entity, light),
        }
    }
}

fn draw_inspector_name(ui: &Ui, ctx: &UiContext, inspector: &mut InspectorData) {
    if !ui.collapsing_header(
        "Tag",
        TreeNodeFlags::DEFAULT_OPEN | TreeNodeFlags::ALLOW_ITEM_OVERLAP,
    ) {
        return;
    }

    if ui.input_text("Name", &mut inspector.name).build() {
        ctx.commands.send(EntityCommand::SetName {
            entity: inspector.entity,
            name: inspector.name.clone(),
        });
    }
}

fn draw_inspector_transform(
    ui: &Ui,
    ctx: &UiContext,
    entity: EntityId,
    transform: &mut TransformData,
) {
    if !ui.collapsing_header(
        "Transform",
        TreeNodeFlags::DEFAULT_OPEN | TreeNodeFlags::ALLOW_ITEM_OVERLAP,
    ) {
        return;
    }

    let edit = draw_transform_fields(ui, transform);

    if edit.activated {
        ctx.commands
            .send(EntityCommand::BeginTransformEdit { entity });
    }

    if edit.changed {
        ctx.commands.send(EntityCommand::SetTransform {
            entity,
            transform: transform.clone(),
        });
    }

    if edit.deactivated {
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

#[derive(Default)]
struct TransformEdit {
    changed: bool,
    activated: bool,
    deactivated: bool,
}

fn draw_transform_fields(ui: &Ui, transform: &mut TransformData) -> TransformEdit {
    ui.group(|| {
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

        TransformEdit {
            changed: translation_edited || rotation_edited || scale_edited,
            activated: translation_activated || rotation_activated || scale_activated,
            deactivated: translation_deactivated || rotation_deactivated || scale_deactivated,
        }
    })
}

fn reset_transform(ctx: &UiContext, entity: EntityId, transform: TransformData) {
    ctx.commands
        .send(EntityCommand::BeginTransformEdit { entity });
    ctx.commands
        .send(EntityCommand::SetTransform { entity, transform });
    ctx.commands
        .send(EntityCommand::EndTransformEdit { entity });
}

fn draw_mesh(ui: &Ui, mesh: &crate::editor::MeshData) {
    if ui.collapsing_header("Mesh", TreeNodeFlags::DEFAULT_OPEN) {
        ui.text(format!("Mesh: {}", mesh.id));
    }
}

fn draw_bounding_box(ui: &Ui, bbox: &crate::editor::BoundingBoxData) {
    if ui.collapsing_header("Bounding Box", TreeNodeFlags::DEFAULT_OPEN) {
        ui.text(format!("Local min: {:?}", bbox.min));
        ui.text(format!("Local max: {:?}", bbox.max));
        ui.separator();
        ui.text(format!("Global min: {:?}", bbox.global_min));
        ui.text(format!("Global max: {:?}", bbox.global_max));
    }
}

fn draw_materials(ui: &Ui, ctx: &mut UiContext, materials: &mut [MaterialData]) {
    if !ui.collapsing_header("Materials", TreeNodeFlags::DEFAULT_OPEN) {
        return;
    }

    for material in materials {
        let id = ui.push_id(material.id.id.index.to_string());
        if ui.collapsing_header(&material.name, TreeNodeFlags::DEFAULT_OPEN) {
            *ctx.material_preview = Some(material.id);
            draw_material(ui, ctx, material);
        }
        id.pop();
    }
}

fn draw_material(ui: &Ui, ctx: &mut UiContext, material: &mut MaterialData) {
    ui.text(&material.name);
    ui.same_line();
    ui.text_disabled(format!("#{}", material.id.id.index));
    ui.separator();

    draw_material_preview(ui, ctx, material);

    let mut changed = false;

    if let Some(_tab_bar) = ui.tab_bar("MaterialTabs") {
        if let Some(_tab) = _tab_bar.begin_tab_item("Surface") {
            changed |= draw_surface(ui, &mut material.desc);
        }

        if let Some(_tab) = _tab_bar.begin_tab_item("Textures") {
            changed |= draw_textures(ui, ctx, &mut material.desc);
        }

        if let Some(_tab) = _tab_bar.begin_tab_item("Transmission") {
            changed |= draw_transmission(ui, &mut material.desc);
        }

        if let Some(_tab) = _tab_bar.begin_tab_item("Volume") {
            changed |= draw_volume(ui, &mut material.desc);
        }

        if let Some(_tab) = _tab_bar.begin_tab_item("Sheen") {
            changed |= draw_sheen(ui, &mut material.desc);
        }

        if let Some(_tab) = _tab_bar.begin_tab_item("Alpha") {
            changed |= draw_alpha(ui, &mut material.desc);
        }
    }

    if changed {
        send_material_update(ctx, material);
    }
}

fn draw_material_preview(ui: &Ui, ctx: &UiContext, material: &MaterialData) {
    let preview = material
        .desc
        .texture(MaterialTextureSlot::BaseColor)
        .and_then(|texture| ctx.textures.asset(texture));

    ui.text("Preview");
    if let Some(texture) = ctx.textures.material_preview() {
        Image::new(texture, [180.0, 180.0]).build(ui);
    } else if let Some(texture) = preview {
        Image::new(texture, [180.0, 180.0]).build(ui);
    } else {
        ui.text_disabled("PBR preview render target not available");
    }
}

fn draw_surface(ui: &Ui, material: &mut MaterialDesc) -> bool {
    let mut changed = false;

    let mut color: [f32; 4] = material.base_color_factor.into();
    if ui
        .color_edit4_config("Base Color", &mut color)
        .inputs(false)
        .build()
    {
        material.base_color_factor = color.into();
        changed = true;
    }

    changed |= Drag::new("Metallic")
        .speed(0.01)
        .range(0.0, 1.0)
        .build(ui, &mut material.metallic_factor);
    changed |= Drag::new("Roughness")
        .speed(0.01)
        .range(0.0, 1.0)
        .build(ui, &mut material.roughness_factor);
    changed |= Drag::new("Normal Scale")
        .speed(0.01)
        .range(0.0, 1.0)
        .build(ui, &mut material.normal_scale);
    changed |= Drag::new("Occlusion")
        .speed(0.01)
        .range(0.0, 1.0)
        .build(ui, &mut material.occlusion_strength);

    let mut emissive: [f32; 4] = material.emissive_factor.into();
    if ui
        .color_edit4_config("Emission", &mut emissive)
        .inputs(false)
        .build()
    {
        material.emissive_factor = emissive.into();
        changed = true;
    }

    changed
}

fn draw_textures(ui: &Ui, ctx: &UiContext, material: &mut MaterialDesc) -> bool {
    let mut changed = false;

    for slot in MaterialTextureSlot::ALL {
        changed |= draw_texture_slot(ui, ctx, material, slot);
    }

    changed
}

fn draw_texture_slot(
    ui: &Ui,
    ctx: &UiContext,
    material: &mut MaterialDesc,
    slot: MaterialTextureSlot,
) -> bool {
    ui.separator();
    ui.text(slot.as_str());

    if material.texture(slot).is_none() {
        ui.text_disabled("No texture assigned");
        return false;
    }

    let mut changed = false;
    let mut enabled = material.slot_get(slot);
    if ui.checkbox("Enabled", &mut enabled) {
        material.slot_set(slot, enabled);
        changed = true;
    }

    if enabled {
        draw_texture_preview(ui, ctx, material, slot);
        changed |= draw_texture_transform(ui, material, slot);
    }

    changed
}

fn draw_texture_preview(
    ui: &Ui,
    ctx: &UiContext,
    material: &MaterialDesc,
    slot: MaterialTextureSlot,
) {
    let Some(texture) = material.texture(slot) else {
        return;
    };

    if let Some(texture_id) = ctx.textures.asset(texture) {
        Image::new(texture_id, [64.0, 64.0]).build(ui);
        ui.same_line();
        ui.text(format!("Texture #{}", texture.id.index));
    } else {
        ui.text_disabled(format!("Texture #{} not available", texture.id.index));
    }
}

fn draw_texture_transform(ui: &Ui, material: &mut MaterialDesc, slot: MaterialTextureSlot) -> bool {
    let Some(transform) = material.uvtransform_mut(slot) else {
        return false;
    };

    if !ui.collapsing_header("UV Transform", TreeNodeFlags::empty()) {
        return false;
    }

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

fn draw_transmission(ui: &Ui, material: &mut MaterialDesc) -> bool {
    let Some(transmission) = material.transmission.as_mut() else {
        ui.text_disabled("Transmission not enabled");
        return false;
    };

    let mut changed = Drag::new("Weight")
        .speed(0.01)
        .range(0.0, 1.0)
        .build(ui, &mut transmission.factor);

    changed |= Drag::new("IOR")
        .speed(0.01)
        .range(1.0, 2.5)
        .build(ui, &mut material.ior);

    changed
}

fn draw_volume(ui: &Ui, material: &mut MaterialDesc) -> bool {
    let Some(volume) = material.volume.as_mut() else {
        ui.text_disabled("Volume not enabled");
        return false;
    };

    let mut changed = false;
    changed |= Drag::new("Thickness")
        .speed(0.01)
        .range(0.0, 1.0)
        .build(ui, &mut volume.thickness_factor);
    changed |= Drag::new("Distance")
        .speed(0.01)
        .range(0.01, 100.0)
        .build(ui, &mut volume.attenuation_distance);
    changed |= ui.color_edit3("Attenuation", &mut volume.attenuation_color);
    changed
}

fn draw_sheen(ui: &Ui, material: &mut MaterialDesc) -> bool {
    let Some(sheen) = material.sheen.as_mut() else {
        ui.text_disabled("Sheen not enabled");
        return false;
    };

    let mut changed = ui.color_edit3("Color", &mut sheen.color_factor);
    changed |= Drag::new("Roughness")
        .speed(0.01)
        .range(0.0, 1.0)
        .build(ui, &mut sheen.roughness_factor);
    changed
}

fn draw_alpha(ui: &Ui, material: &mut MaterialDesc) -> bool {
    let mut changed = false;

    match material.alpha_mode {
        crate::assets::material_desc::AlphaMode::Opaque => ui.text("Mode: Opaque"),
        crate::assets::material_desc::AlphaMode::Mask {
            ref mut alpha_cutoff,
        } => {
            ui.text("Mode: Mask");
            changed |= Drag::new("Cutoff")
                .speed(0.01)
                .range(0.0, 1.0)
                .build(ui, alpha_cutoff);
        }
        crate::assets::material_desc::AlphaMode::Blend => ui.text("Mode: Blend"),
    }

    ui.text("Select mode:");
    if ui.small_button("Opaque") {
        material.alpha_mode = crate::assets::material_desc::AlphaMode::Opaque;
        changed = true;
    }
    ui.same_line();
    if ui.small_button("Mask") {
        material.alpha_mode = crate::assets::material_desc::AlphaMode::mask_default();
        changed = true;
    }
    ui.same_line();
    if ui.small_button("Blend") {
        material.alpha_mode = crate::assets::material_desc::AlphaMode::Blend;
        changed = true;
    }

    changed
}

fn send_material_update(ctx: &UiContext, material: &MaterialData) {
    ctx.commands.send(AssetCommand::UpdateMaterial {
        id: material.id,
        desc: material.desc.clone(),
    });
}

fn draw_light(ui: &Ui, ctx: &UiContext, entity: EntityId, light: &mut LightData) {
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
