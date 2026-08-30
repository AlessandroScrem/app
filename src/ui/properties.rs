use super::ui_layer::{Layer, UiContext};
use crate::editor::{
    EditValue, EditorCommand, EditorEdit, EntityId, InspectorData, LightData, TransformData,
};
use imgui::*;

impl UiContext<'_> {
    fn edit(&self, entity: EntityId) -> Option<&EditValue> {
        self.edit
            .as_ref()
            .filter(|edit| edit.key == entity)
            .map(|edit| &edit.value)
    }

    pub fn is_editing(&self, entity: EntityId) -> bool {
        self.edit.as_ref().is_some_and(|edit| edit.key == entity)
    }

    pub fn begin_edit(&mut self, entity: EntityId, value: EditValue) {
        *self.edit = Some(EditorEdit::new(entity, value));
    }

    pub fn end_edit(&mut self) {
        *self.edit = None;
    }
}

impl EditValue {
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Name(value) => Some(value),
            _ => None,
        }
    }

    pub fn transform(&self) -> Option<&TransformData> {
        match self {
            Self::Transform(value) => Some(value),
            _ => None,
        }
    }
    #[allow(dead_code)]
    pub fn light(&self) -> Option<&LightData> {
        match self {
            Self::Light(value) => Some(value),
            _ => None,
        }
    }
}

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

    if !ctx.is_editing(inspector.entity) {
        ctx.end_edit();
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
    if !ui.collapsing_header(
        "Tag",
        TreeNodeFlags::DEFAULT_OPEN | TreeNodeFlags::ALLOW_ITEM_OVERLAP,
    ) {
        return;
    }

    let mut name = ctx
        .edit(inspector.entity)
        .and_then(EditValue::name)
        .unwrap_or(&inspector.name.clone())
        .to_owned();

    let edited = ui.input_text("Name", &mut name).build();
    let activated = ui.is_item_activated();
    let deactivated = ui.is_item_deactivated();

    if activated {
        ctx.begin_edit(inspector.entity, EditValue::Name(name.clone()));
    }

    if edited {
        ctx.connection.commands.send(EditorCommand::SetName {
            entity: inspector.entity,
            name: name.clone(),
        });
    }
    
    if deactivated {
        ctx.end_edit();
    }
}

fn draw_inspector_transform(ui: &Ui, ctx: &mut UiContext, inspector: &InspectorData) {
    if ui.collapsing_header(
        "Transform",
        TreeNodeFlags::DEFAULT_OPEN | TreeNodeFlags::ALLOW_ITEM_OVERLAP,
    ) {
        return;
    }

    let mut transform = ctx
        .edit(inspector.entity)
        .and_then(EditValue::transform)
        .unwrap_or(&inspector.transform.clone())
        .to_owned();

    ui.group(|| {
        Drag::new("Translation")
            .speed(0.1)
            .build_array(ui, &mut transform.translation);
        Drag::new("Rotation")
            .speed(0.01)
            .build_array(ui, &mut transform.rotation);
        Drag::new("Scale")
            .speed(0.1)
            .build_array(ui, &mut transform.scale);
    });

    let edited = ui.is_item_edited();
    let activated = ui.is_item_activated();
    let deactivated = ui.is_item_deactivated_after_edit();

    if activated {
        ctx.begin_edit(inspector.entity, EditValue::Transform(transform.clone()));

        ctx.connection
            .commands
            .send(EditorCommand::BeginTransformEdit {
                entity: inspector.entity,
            });
    }

    if edited {
        ctx.connection.commands.send(EditorCommand::SetTransform {
            entity: inspector.entity,
            transform: transform.clone(),
        });
    }

    if deactivated {
        ctx.connection
            .commands
            .send(EditorCommand::EndTransformEdit {
                entity: inspector.entity,
            });
        ctx.end_edit();
    }

    ui.separator();
    if ui.small_button("Reset Transform") {
        let identity = TransformData {
            translation: [0.0; 3],
            rotation: [0.0; 3],
            scale: [1.0; 3],
        };
        reset_transform(ctx, inspector.entity, identity);
    }

    ui.same_line();
    if ui.small_button("Reset Position") {
        transform.translation = [0.0; 3];
        reset_transform(ctx, inspector.entity, transform.clone());
    }

    ui.same_line();
    if ui.small_button("Reset Rotation") {
        transform.rotation = [0.0; 3];
        reset_transform(ctx, inspector.entity, transform.clone());
    }
}

fn reset_transform(ctx: &mut UiContext, entity: EntityId, transform: TransformData) {
    ctx.connection
        .commands
        .send(EditorCommand::BeginTransformEdit { entity });

    ctx.connection
        .commands
        .send(EditorCommand::SetTransform { entity, transform });

    ctx.connection
        .commands
        .send(EditorCommand::EndTransformEdit { entity });

    ctx.end_edit();
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

    ui.group(|| {
        Drag::new("Position")
            .speed(0.1)
            .build_array(ui, &mut light.position);
        ui.color_edit3("Color", &mut light.color);
        ui.checkbox("Enabled", &mut light.enabled);
        ui.checkbox("Directional", &mut light.directional);
        ui.checkbox("Cast Shadow", &mut light.cast_shadow);
        if light.cast_shadow {
            ui.checkbox("Frustum", &mut light.frustum);
        }
    });

    let activated = ui.is_item_activated();
    let edited = ui.is_item_edited();
    let deactivated = ui.is_item_deactivated_after_edit();

    if activated {
        ctx.begin_edit(entity, EditValue::Light(source.clone()));
    }

    if edited {
        println!("Light edited");
        ctx.connection
            .commands
            .send(EditorCommand::SetLight { entity, light });
    }

    if deactivated {
        ctx.end_edit();
    }
}
