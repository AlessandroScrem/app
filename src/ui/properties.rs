use super::ui_layer::{Layer, UiContext};
use crate::editor::{
    EditValue, EditorCommand, EditorEdit, InspectorData, LightData, TransformData,
};
use imgui::*;

pub struct PropertyUi;
impl Layer for PropertyUi {
    fn build(&mut self, ui: &Ui, ctx: &mut UiContext) {
        let window = ui
            .window("Properties")
            .size([420.0, 560.0], Condition::FirstUseEver);

        window.build(|| draw_inspector(ui, ctx));
    }
}

fn draw_inspector(ui: &Ui, ctx: &mut UiContext) {
    let Some(inspector) = ctx.inspector else {
        ui.text(format!("{} entities selected", ctx.selection.len()));
        for entity in ctx.selection.iter() {
            ui.text(format!("Entity: {}", entity));
        }
        return;
    };

    ui.text(format!("{}  [#{}]", inspector.name, inspector.entity));
    ui.separator();

    if ctx
        .edit
        .as_ref()
        .is_some_and(|edit| edit.key != inspector.entity)
    {
        *ctx.edit = None;
    }

    draw_inspector_name(ui, ctx, inspector);

    draw_inspector_transform(ui, ctx, inspector);

    if let Some(mesh) = &inspector.mesh {
        if ui.collapsing_header("Mesh", TreeNodeFlags::DEFAULT_OPEN) {
            ui.text(format!("Mesh: {}", mesh.id));
        }
    }
    if let Some(bbox) = &inspector.bounding_box {
        if ui.collapsing_header("Bounding Box", TreeNodeFlags::DEFAULT_OPEN) {
            ui.text(format!("Local min: {:?}", bbox.min));
            ui.text(format!("Local max: {:?}", bbox.max));
            ui.separator();
            ui.text(format!("Global min: {:?}", bbox.global_min));
            ui.text(format!("Global max: {:?}", bbox.global_max));
        }
    }
    if let Some(light) = &inspector.light {
        draw_light(ui, ctx, inspector.entity, light);
    }
}

fn draw_inspector_name(ui: &Ui, ctx: &mut UiContext, inspector: &InspectorData) {
    if ui.collapsing_header(
        "Tag",
        TreeNodeFlags::DEFAULT_OPEN | TreeNodeFlags::ALLOW_ITEM_OVERLAP,
    ) {
        let mut name = ctx
            .edit
            .as_ref()
            .filter(|edit| edit.key == inspector.entity)
            .and_then(|edit| match &edit.value {
                EditValue::Name(name) => Some(name.clone()),
                _ => None,
            })
            .unwrap_or_else(|| inspector.name.clone());

        let changed = ui.input_text("Name", &mut name).build();
        let active = ui.is_item_active();
        let activated = ui.is_item_activated();
        let deactivated = ui.is_item_deactivated();

        if activated {
            *ctx.edit = Some(EditorEdit::new(
                inspector.entity,
                EditValue::Name(name.clone()),
            ));
        }

        if active || changed {
            ctx.connection.commands.send(EditorCommand::SetName {
                entity: inspector.entity,
                name: name.clone(),
            });
        }
        if deactivated {
            *ctx.edit = None;
        }
    }
}

fn draw_inspector_transform(ui: &Ui, ctx: &mut UiContext, inspector: &InspectorData) {
    if ui.collapsing_header(
        "Transform",
        TreeNodeFlags::DEFAULT_OPEN | TreeNodeFlags::ALLOW_ITEM_OVERLAP,
    ) {
        let mut transform = ctx
            .edit
            .as_ref()
            .filter(|edit| edit.key == inspector.entity)
            .and_then(|edit| match &edit.value {
                EditValue::Transform(transform) => Some(transform.clone()),
                _ => None,
            })
            .unwrap_or_else(|| inspector.transform.clone());

        let translation_changed = Drag::new("Translation")
            .speed(0.1)
            .build_array(ui, &mut transform.translation);
        let translation_active = ui.is_item_active();
        let translation_activated = ui.is_item_activated();
        let translation_deactivated = ui.is_item_deactivated_after_edit();

        let rotation_changed = Drag::new("Rotation")
            .speed(0.01)
            .build_array(ui, &mut transform.rotation);
        let rotation_active = ui.is_item_active();
        let rotation_activated = ui.is_item_activated();
        let rotation_deactivated = ui.is_item_deactivated_after_edit();

        let scale_changed = Drag::new("Scale")
            .speed(0.1)
            .build_array(ui, &mut transform.scale);
        let scale_active = ui.is_item_active();
        let scale_activated = ui.is_item_activated();
        let scale_deactivated = ui.is_item_deactivated_after_edit();

        let changed = translation_changed || rotation_changed || scale_changed;
        let active = translation_active || rotation_active || scale_active;
        let activated = translation_activated || rotation_activated || scale_activated;
        let deactivated = translation_deactivated || rotation_deactivated || scale_deactivated;
        let editing_same_entity = ctx
            .edit
            .as_ref()
            .is_some_and(|edit| edit.key == inspector.entity);

        if activated || (active && !editing_same_entity) {
            ctx.connection
                .commands
                .send(EditorCommand::BeginTransformEdit {
                    entity: inspector.entity,
                });
        }

        if active || changed || editing_same_entity {
            *ctx.edit = Some(EditorEdit::new(
                inspector.entity,
                EditValue::Transform(transform.clone()),
            ));
            if changed {
                ctx.connection.commands.send(EditorCommand::SetTransform {
                    entity: inspector.entity,
                    transform: transform.clone(),
                });
            }
        }

        if deactivated && editing_same_entity {
            ctx.connection
                .commands
                .send(EditorCommand::EndTransformEdit {
                    entity: inspector.entity,
                });
            *ctx.edit = None;
        }

        ui.separator();
        if ui.small_button("Reset Transform") {
            let identity = TransformData {
                translation: [0.0; 3],
                rotation: [0.0; 3],
                scale: [1.0; 3],
            };
            ctx.connection
                .commands
                .send(EditorCommand::BeginTransformEdit {
                    entity: inspector.entity,
                });
            ctx.connection.commands.send(EditorCommand::SetTransform {
                entity: inspector.entity,
                transform: identity,
            });
            ctx.connection
                .commands
                .send(EditorCommand::EndTransformEdit {
                    entity: inspector.entity,
                });
            *ctx.edit = None;
        }
        ui.same_line();
        if ui.small_button("Reset Position") {
            transform.translation = [0.0; 3];
            ctx.connection
                .commands
                .send(EditorCommand::BeginTransformEdit {
                    entity: inspector.entity,
                });
            ctx.connection.commands.send(EditorCommand::SetTransform {
                entity: inspector.entity,
                transform: transform.clone(),
            });
            ctx.connection
                .commands
                .send(EditorCommand::EndTransformEdit {
                    entity: inspector.entity,
                });
            *ctx.edit = None;
        }
        ui.same_line();
        if ui.small_button("Reset Rotation") {
            transform.rotation = [0.0; 3];
            ctx.connection
                .commands
                .send(EditorCommand::BeginTransformEdit {
                    entity: inspector.entity,
                });
            ctx.connection.commands.send(EditorCommand::SetTransform {
                entity: inspector.entity,
                transform: transform.clone(),
            });
            ctx.connection
                .commands
                .send(EditorCommand::EndTransformEdit {
                    entity: inspector.entity,
                });
            *ctx.edit = None;
        }
    }
}

fn draw_light(ui: &Ui, ctx: &mut UiContext, entity: u64, source: &LightData) {
    if !ui.collapsing_header("Light", TreeNodeFlags::DEFAULT_OPEN) {
        return;
    }

    let mut light = ctx
        .edit
        .as_ref()
        .filter(|edit| edit.key == entity)
        .and_then(|edit| match &edit.value {
            EditValue::Light(light) => Some(light.clone()),
            _ => None,
        })
        .unwrap_or_else(|| source.clone());

    let position_changed = Drag::new("Position")
        .speed(0.1)
        .build_array(ui, &mut light.position);
    let position_active = ui.is_item_active();
    let position_activated = ui.is_item_activated();
    let position_deactivated = ui.is_item_deactivated_after_edit();

    let color_changed = ui.color_edit3("Color", &mut light.color);
    let color_active = ui.is_item_active();
    let color_activated = ui.is_item_activated();
    let color_deactivated = ui.is_item_deactivated_after_edit();

    let enabled_changed = ui.checkbox("Enabled", &mut light.enabled);
    let enabled_active = ui.is_item_active();
    let enabled_activated = ui.is_item_activated();
    let enabled_deactivated = ui.is_item_deactivated_after_edit();

    let directional_changed = ui.checkbox("Directional", &mut light.directional);
    let directional_active = ui.is_item_active();
    let directional_activated = ui.is_item_activated();
    let directional_deactivated = ui.is_item_deactivated_after_edit();

    let cast_shadow_changed = ui.checkbox("Cast Shadow", &mut light.cast_shadow);
    let cast_shadow_active = ui.is_item_active();
    let cast_shadow_activated = ui.is_item_activated();
    let cast_shadow_deactivated = ui.is_item_deactivated_after_edit();

    let mut frustum_changed = false;
    let mut frustum_active = false;
    let mut frustum_activated = false;
    let mut frustum_deactivated = false;

    if light.cast_shadow {
        frustum_changed = ui.checkbox("Frustum", &mut light.frustum);
        frustum_active = ui.is_item_active();
        frustum_activated = ui.is_item_activated();
        frustum_deactivated = ui.is_item_deactivated_after_edit();
    }

    let changed = position_changed
        || color_changed
        || enabled_changed
        || directional_changed
        || cast_shadow_changed
        || frustum_changed;

    let active = position_active
        || color_active
        || enabled_active
        || directional_active
        || cast_shadow_active
        || frustum_active;

    let activated = position_activated
        || color_activated
        || enabled_activated
        || directional_activated
        || cast_shadow_activated
        || frustum_activated;

    let deactivated = position_deactivated
        || color_deactivated
        || enabled_deactivated
        || directional_deactivated
        || cast_shadow_deactivated
        || frustum_deactivated;

    let editing_same_entity = ctx.edit.as_ref().is_some_and(|edit| edit.key == entity);

    if activated || (active && !editing_same_entity) {
        *ctx.edit = Some(EditorEdit::new(entity, EditValue::Light(source.clone())));
    }

    if active || changed || editing_same_entity {
        *ctx.edit = Some(EditorEdit::new(entity, EditValue::Light(light.clone())));

        if changed {
            ctx.connection
                .commands
                .send(EditorCommand::SetLight { entity, light });
        }
    }

    if deactivated && editing_same_entity {
        *ctx.edit = None;
    }
}
