use imgui::{Condition, MouseButton, Ui, WindowFlags};

use crate::editor::EditorCommand;
use crate::ui::ui_layer::{Layer, UiContext, WindowAction};

#[derive(Default)]
pub struct TitleBarUi;

impl Layer for TitleBarUi {
    fn update(&mut self, _commands: &super::ui_commands::UiCommands) {}

    fn build(&mut self, ui: &Ui, ctx: &mut UiContext) {
        let width = ui.io().display_size[0];

        ui.window("##TitleBar")
            .position([0.0, 0.0], Condition::Always)
            .size([width, 32.0], Condition::Always)
            .flags(
                WindowFlags::NO_DECORATION
                    | WindowFlags::NO_SAVED_SETTINGS
                    | WindowFlags::NO_SCROLLBAR
                    | WindowFlags::NO_MOVE
                    | WindowFlags::NO_RESIZE,
            )
            .build(|| {
                ui.text("App");
                ui.same_line();

                let drag_width = (width - 180.0).max(0.0);
                ui.invisible_button("##WindowDrag", [drag_width, 24.0]);
                if ui.is_item_active() && ui.is_mouse_dragging(MouseButton::Left) {
                    *ctx.window_action = Some(WindowAction::Drag);
                }

                ui.same_line();
                if ui.button_with_size("—", [32.0, 24.0]) {
                    *ctx.window_action = Some(WindowAction::Minimize);
                }
                ui.same_line();
                if ui.button_with_size("□", [32.0, 24.0]) {
                    *ctx.window_action = Some(WindowAction::ToggleMaximize);
                }
                ui.same_line();
                if ui.button_with_size("×", [32.0, 24.0]) {
                    ctx.commands.send(EditorCommand::Exit);
                }
            });
    }
}
