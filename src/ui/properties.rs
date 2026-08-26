use super::ui_layer::{Layer, UiContext};
use crate::editor::{EditorCommand, LightData, TransformData};
use imgui::*;

pub struct PropertyUi;
impl Layer for PropertyUi {
    fn build(&mut self, ui: &Ui, ctx: &mut UiContext) {
        let focus = *ctx.focus_properties;
        *ctx.focus_properties = false;

        ui.window("Properties")
            .size([420.0, 560.0], Condition::FirstUseEver)
            .build(|| draw_inspector(ui, ctx));

        if focus {
            ui.set_window_focus(Some("Properties"));
        }
    }
}

fn draw_inspector(ui: &Ui, ctx: &mut UiContext) {
    let Some(inspector) = ctx.inspector else {
        ui.text("No entity selected");
        return;
    };
    ui.text(format!("{}  [#{}]", inspector.name, inspector.entity));
    ui.separator();

    if ctx
        .transform_edit
        .as_ref()
        .is_some_and(|(entity, _)| *entity != inspector.entity)
    {
        *ctx.transform_edit = None;
    }

    let mut transform = ctx
        .transform_edit
        .as_ref()
        .filter(|(entity, _)| *entity == inspector.entity)
        .map(|(_, t)| t.clone())
        .unwrap_or_else(|| inspector.transform.clone());

    if ui.collapsing_header(
        "Tag",
        TreeNodeFlags::DEFAULT_OPEN | TreeNodeFlags::ALLOW_ITEM_OVERLAP,
    ) {
        let mut name = inspector.name.clone();
        if ui.input_text("Name", &mut name).build() && name != inspector.name {
            ctx.connection.commands.send(EditorCommand::SetName {
                entity: inspector.entity,
                name,
            });
        }
    }

    if ui.collapsing_header(
        "Transform",
        TreeNodeFlags::DEFAULT_OPEN | TreeNodeFlags::ALLOW_ITEM_OVERLAP,
    ) {
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
            .transform_edit
            .as_ref()
            .is_some_and(|(entity, _)| *entity == inspector.entity);

        if activated || (active && !editing_same_entity) {
            ctx.connection
                .commands
                .send(EditorCommand::BeginTransformEdit {
                    entity: inspector.entity,
                });
        }

        if active || changed || editing_same_entity {
            *ctx.transform_edit = Some((inspector.entity, transform.clone()));
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
            *ctx.transform_edit = None;
        }

        ui.separator();
        if ui.small_button("Reset Transform") {
            let identity = TransformData {
                translation: [0.0; 3],
                rotation: [0.0; 3],
                scale: [1.0; 3],
            };
            ctx.connection.commands.send(EditorCommand::BeginTransformEdit {
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
            *ctx.transform_edit = None;
        }
        ui.same_line();
        if ui.small_button("Reset Position") {
            transform.translation = [0.0; 3];
            ctx.connection.commands.send(EditorCommand::BeginTransformEdit {
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
            *ctx.transform_edit = None;
        }
        ui.same_line();
        if ui.small_button("Reset Rotation") {
            transform.rotation = [0.0; 3];
            ctx.connection.commands.send(EditorCommand::BeginTransformEdit {
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
            *ctx.transform_edit = None;
        }
    }

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

fn draw_light(ui: &Ui, ctx: &mut UiContext, entity: u64, source: &LightData) {
    if !ui.collapsing_header("Light", TreeNodeFlags::DEFAULT_OPEN) {
        return;
    }
    let mut light = source.clone();
    let mut changed = false;
    changed |= Drag::new("Position")
        .speed(0.1)
        .build_array(ui, &mut light.position);
    changed |= ui.color_edit3("Color", &mut light.color);
    changed |= ui.checkbox("Enabled", &mut light.enabled);
    changed |= ui.checkbox("Directional", &mut light.directional);
    changed |= ui.checkbox("Cast Shadow", &mut light.cast_shadow);
    if light.cast_shadow {
        changed |= ui.checkbox("Frustum", &mut light.frustum);
    }
    if changed {
        ctx.connection.commands.send(EditorCommand::SetLight { entity, light });
    }
}
