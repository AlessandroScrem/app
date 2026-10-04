use imgui::{Drag, TreeNodeFlags, Ui};

use crate::{
    editor::{EntityCommand, EntityId, TransformData},
    ui::UiContext,
};

#[derive(Default)]
struct TransformEdit {
    changed: bool,
    activated: bool,
    deactivated: bool,
}

pub fn draw(ui: &Ui, ctx: &UiContext, entity: EntityId, transform: &mut TransformData) {
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
