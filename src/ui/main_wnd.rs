use super::*;
use imgui::*;

pub struct ViewportUi {}

impl Layer for ViewportUi {
    fn build(&mut self, ui: &Ui, ctx: &mut UiContext) {
        // Skip if ui capture...
        if ui.io().want_capture_mouse || ui.io().want_capture_keyboard {
            return;
        }

        match ctx.editor_interaction {
            EditorInteraction::Selecting { start, current } => {
                let start = to_logical(ui, (*start).into());
                let current = to_logical(ui, (*current).into());

                ui.get_foreground_draw_list()
                    .add_rect(start, current, [1.0, 0.0, 0.0, 1.0])
                    .thickness(1.0)
                    .build();
            }
            _ => {}
        }
    }
}

// Convert winit PhisicalCoordinate to LogicalCoordinate
fn to_logical(ui: &Ui, pos: [f32; 2]) -> [f32; 2] {
    let dpi_scale = ui.io().display_framebuffer_scale;
    let x = (pos[0] / dpi_scale[0]).round();
    let y = (pos[1] / dpi_scale[1]).round();

    [x, y]
}
