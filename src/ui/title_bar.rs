use imgui::{Condition, MouseButton, Ui, WindowFlags};
use winit::window::ResizeDirection;

use crate::editor::EditorCommand;
use crate::ui::ui_layer::{Layer, UiContext, WindowAction};

const TITLE_BAR_HEIGHT: f32 = 32.0;
const RESIZE_BORDER: f32 = 6.0;

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

        ui.window("##WindowResizeHandles")
            .position([0.0, 0.0], Condition::Always)
            .size([width, height], Condition::Always)
            .flags(
                WindowFlags::NO_DECORATION
                    | WindowFlags::NO_SAVED_SETTINGS
                    | WindowFlags::NO_SCROLLBAR
                    | WindowFlags::NO_MOVE
                    | WindowFlags::NO_RESIZE
                    | WindowFlags::NO_BACKGROUND,
            )
            .build(|| {
                let edge = RESIZE_BORDER;
                let horizontal = (width - 2.0 * edge).max(0.0);
                let vertical = (height - 2.0 * edge).max(0.0);

                resize_handle(
                    ui,
                    "##ResizeNorthWest",
                    [0.0, 0.0],
                    [edge, edge],
                    ResizeDirection::NorthWest,
                    ctx,
                );
                resize_handle(
                    ui,
                    "##ResizeNorth",
                    [edge, 0.0],
                    [horizontal, edge],
                    ResizeDirection::North,
                    ctx,
                );
                resize_handle(
                    ui,
                    "##ResizeNorthEast",
                    [width - edge, 0.0],
                    [edge, edge],
                    ResizeDirection::NorthEast,
                    ctx,
                );
                resize_handle(
                    ui,
                    "##ResizeWest",
                    [0.0, edge],
                    [edge, vertical],
                    ResizeDirection::West,
                    ctx,
                );
                resize_handle(
                    ui,
                    "##ResizeEast",
                    [width - edge, edge],
                    [edge, vertical],
                    ResizeDirection::East,
                    ctx,
                );
                resize_handle(
                    ui,
                    "##ResizeSouthWest",
                    [0.0, height - edge],
                    [edge, edge],
                    ResizeDirection::SouthWest,
                    ctx,
                );
                resize_handle(
                    ui,
                    "##ResizeSouth",
                    [edge, height - edge],
                    [horizontal, edge],
                    ResizeDirection::South,
                    ctx,
                );
                resize_handle(
                    ui,
                    "##ResizeSouthEast",
                    [width - edge, height - edge],
                    [edge, edge],
                    ResizeDirection::SouthEast,
                    ctx,
                );
            });
    }
}

fn resize_handle(
    ui: &Ui,
    id: &str,
    position: [f32; 2],
    size: [f32; 2],
    direction: ResizeDirection,
    ctx: &mut UiContext,
) {
    ui.set_cursor_pos(position);
    ui.invisible_button(id, size);
    if ui.is_item_clicked() {
        *ctx.window_action = Some(WindowAction::Resize(direction));
    }
}
