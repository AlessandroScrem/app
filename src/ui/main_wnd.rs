use crate::editor::EditorCommand;

use super::ui_layer::{Layer, UiContext};

use imgui::*;

#[derive(Debug, Default, Clone)]
pub struct ViewportUi {
    click_pos: Option<[f32; 2]>,
}

impl Layer for ViewportUi {
    fn build(&mut self, ui: &Ui, ctx: &mut UiContext) {
        if ui.io().want_capture_mouse {
            return;
        }

        match self.click_pos {
            None => {
                if ui.is_mouse_clicked(MouseButton::Left) && ui.is_key_down(Key::LeftCtrl) {
                    self.click_pos = Some(ui.io().mouse_pos);
                }
            }
            Some(start) => {
                let current = ui.io().mouse_pos;
                if ui.is_mouse_dragging(MouseButton::Left) && ui.is_key_down(Key::LeftCtrl) {
                    ui.get_foreground_draw_list()
                        .add_rect(start, current, [1.0, 0.0, 0.0, 1.0])
                        .thickness(1.0)
                        .build();
                }

                if ui.is_mouse_released(MouseButton::Left) {
                    let scale = ui.io().display_framebuffer_scale;
                    let start = [start[0] * scale[0], start[1] * scale[1]];
                    let current = [current[0] * scale[0], current[1] * scale[1]];

                    let pos = (
                        start[0].min(current[0]) as u32,
                        start[1].min(current[1]) as u32,
                    );
                    let width = (start[0] - current[0]).abs() as u32;
                    let height = (start[1] - current[1]).abs() as u32;
                    let size = (width, height);

                    ctx.commands.send(EditorCommand::DragSelection(pos, size));
                    self.click_pos = None;
                }
            }
        }
    }
}
