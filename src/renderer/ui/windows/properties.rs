use super::*;

pub fn draw_window_properties(ui: &imgui::Ui, ctx: &mut UiContext) {
    ui.window("Properties")
        .size([300.0, 300.0], Condition::FirstUseEver)
        .build(|| {
            inspector::draw_entity_inspector(ui, ctx);
        });
}
