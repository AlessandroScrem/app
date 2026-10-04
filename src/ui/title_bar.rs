use imgui::{Condition, MouseButton, MouseCursor, StyleColor, Ui, WindowFlags};

use crate::editor::EditorCommand;
use crate::ui::ui_layer::{Layer, UiContext, WindowAction};

const TITLE_BAR_HEIGHT: f32 = 36.0;
const WINDOW_BUTTON_SIZE: [f32; 2] = [36.0, 28.0];

#[derive(Default)]
pub struct TitleBarUi;

impl Layer for TitleBarUi {
    fn update(&mut self, _commands: &super::ui_commands::UiCommands) {}

    fn build(&mut self, ui: &Ui, ctx: &mut UiContext) {
        let [width, height] = ui.io().display_size;
        if width <= 0.0 || height <= 0.0 {
            return;
        }

        ui.window("##TitleBar")
            .position([0.0, 0.0], Condition::Always)
            .size([width, TITLE_BAR_HEIGHT], Condition::Always)
            .flags(
                WindowFlags::NO_DECORATION
                    | WindowFlags::NO_SAVED_SETTINGS
                    | WindowFlags::NO_SCROLLBAR
                    | WindowFlags::NO_MOVE
                    | WindowFlags::NO_RESIZE,
            )
            .build(|| {
                ui.set_cursor_pos([12.0, 7.0]);
                ui.text_colored([0.82, 0.84, 0.88, 1.0], "App");
                ui.same_line();

                let drag_width = (width - 180.0).max(0.0);
                ui.invisible_button("##WindowDrag", [drag_width, 28.0]);

                if ui.is_item_hovered() {
                    ui.set_mouse_cursor(Some(MouseCursor::Arrow));
                }

                if ui.is_item_hovered()
                    && ui.is_mouse_double_clicked(MouseButton::Left)
                {
                    *ctx.window_action = Some(WindowAction::ToggleMaximize);
                } else if ui.is_item_active()
                    && ui.is_mouse_dragging(MouseButton::Left)
                {
                    *ctx.window_action = Some(WindowAction::Drag);
                }

                ui.same_line();

                window_button(ui, "—", false, || {
                    *ctx.window_action = Some(WindowAction::Minimize);
                });
                ui.same_line();

                window_button(ui, "□", false, || {
                    *ctx.window_action = Some(WindowAction::ToggleMaximize);
                });
                ui.same_line();

                window_button(ui, "×", true, || {
                    ctx.commands.send(EditorCommand::Exit);
                });
            });
    }
}

fn window_button(
    ui: &Ui,
    label: &str,
    close: bool,
    action: impl FnOnce(),
) {
    let button = ui.push_style_color(StyleColor::Button, [0.0, 0.0, 0.0, 0.0]);
    let hovered = if close {
        [0.78, 0.17, 0.17, 1.0]
    } else {
        [0.20, 0.22, 0.25, 1.0]
    };
    let hover = ui.push_style_color(StyleColor::ButtonHovered, hovered);
    let active = ui.push_style_color(StyleColor::ButtonActive, hovered);

    if ui.button_with_size(label, WINDOW_BUTTON_SIZE) {
        action();
    }

    active.pop();
    hover.pop();
    button.pop();
}
