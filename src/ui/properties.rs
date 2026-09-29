use super::ui_layer::{Layer, UiContext};
use crate::editor::EntityCommand;
use crate::editor::{EntityId, InspectorData, InspectorSection, LightData, TransformData};
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
            ui.image(imgui::TextureId::new(texture.raw()), [200.0, 200.0]);
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
