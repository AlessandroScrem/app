use imgui::{Condition, MouseButton, MouseCursor, StyleColor, StyleVar, Ui, WindowFlags};

use crate::editor::{AssetCommand, EditorCommand, SceneCommand, SceneSettingsData};
use crate::ui::menu_bar::{file_open, file_save, FileFilter};
use crate::ui::ui_layer::{Layer, UiContext, WindowAction};

const TOP_BAR_HEIGHT: f32 = 36.0;
const WINDOW_BUTTON_SIZE: [f32; 2] = [36.0, 28.0];
const WINDOW_BUTTONS_WIDTH: f32 = WINDOW_BUTTON_SIZE[0] * 3.0;
const DRAG_START: f32 = 180.0;

const ICON_MINIMIZE: &str = "\u{EABA}";
const ICON_MAXIMIZE: &str = "\u{EAB9}";
const ICON_CLOSE: &str = "\u{EAB8}";

#[derive(Default)]
pub struct TopBarUi {
    scene_settings: SceneSettingsData,
}

impl Layer for TopBarUi {
    fn update(&mut self, commands: &super::ui_commands::UiCommands) {
        self.scene_settings = commands.scene_settings().clone();
    }

    fn build(&mut self, ui: &Ui, ctx: &mut UiContext) {
        let [width, height] = ui.io().display_size;
        if width <= 0.0 || height <= 0.0 {
            return;
        }

        ui.window("##TopBar")
            .position([0.0, 0.0], Condition::Always)
            .size([width, TOP_BAR_HEIGHT], Condition::Always)
            .flags(
                WindowFlags::NO_DECORATION
                    | WindowFlags::NO_SAVED_SETTINGS
                    | WindowFlags::NO_SCROLLBAR
                    | WindowFlags::NO_MOVE
                    | WindowFlags::NO_RESIZE,
            )
            .build(|| {
                let menu_width = 180.0_f32.min(width);

                ui.set_cursor_pos([8.0, 4.0]);
                if let Some(_menu) = ui.begin_menu("File") {
                    if ui.menu_item("New") {
                        ctx.commands.send(EditorCommand::Scene(SceneCommand::Clear));
                    }
                    if ui.menu_item("Open Scene") {
                        if let Some(path) = file_open(FileFilter::Json) {
                            ctx.commands
                                .send(EditorCommand::Scene(SceneCommand::Open(path)));
                        }
                    }
                    if ui.menu_item("Save As..") {
                        if let Some(path) = file_save(FileFilter::Json) {
                            ctx.commands
                                .send(EditorCommand::Scene(SceneCommand::SaveAs(path)));
                        }
                    }
                    if ui.menu_item("Save") {
                        ctx.commands.send(EditorCommand::Scene(SceneCommand::Save));
                    }
                    ui.separator();
                    if ui.menu_item("Load Gltf") {
                        if let Some(path) = file_open(FileFilter::Gltf) {
                            ctx.commands
                                .send(EditorCommand::Asset(AssetCommand::LoadGltf(path)));
                        }
                    }
                    if ui.menu_item("Add Ibl") {
                        if let Some(path) = file_open(FileFilter::Hdr) {
                            ctx.commands
                                .send(EditorCommand::Asset(AssetCommand::AddIbl(path)));
                        }
                    }
                    if ui.menu_item("Clear Scene") {
                        ctx.commands.send(EditorCommand::Scene(SceneCommand::Clear));
                    }
                    ui.separator();
                    if ui.menu_item("Exit") {
                        ctx.commands.send(EditorCommand::Exit);
                    }
                    ui.separator();
                    ui.menu("Recent Files", || {
                        for (name, path) in &self.scene_settings.recent {
                            if ui.menu_item(name) {
                                ctx.commands
                                    .send(EditorCommand::Scene(SceneCommand::Open(path.into())));
                            }
                        }
                        if self.scene_settings.recent.is_empty() {
                            ui.text_disabled("No recent files");
                        }
                    });
                }

                ui.same_line();
                if let Some(_menu) = ui.begin_menu("Edit") {
                    ui.menu_item("Undo");
                    ui.menu_item("Redo");
                }

                ui.same_line();
                if let Some(_menu) = ui.begin_menu("View") {
                    ui.menu_item("Show Stats");
                }

                let title_size = ui.calc_text_size(ctx.window_title);
                let title_x = ((width - title_size[0]) * 0.5).max(0.0);
                ui.set_cursor_pos([title_x, (TOP_BAR_HEIGHT - title_size[1]) * 0.5]);
                ui.text_colored([0.82, 0.84, 0.88, 1.0], ctx.window_title);

                let buttons_start = (width - WINDOW_BUTTONS_WIDTH).max(0.0);
                let drag_left = menu_width + 8.0;
                let drag_right = title_x - 8.0;
                let drag_width = (drag_right - drag_left).max(0.0);

                if drag_width > 0.0 {
                    ui.set_cursor_pos([drag_left, 4.0]);
                    ui.invisible_button("##WindowDragLeft", [drag_width, 28.0]);
                    handle_drag(ui, ctx);
                }

                let title_right = title_x + title_size[0] + 8.0;
                let drag_width = (buttons_start - title_right).max(0.0);
                if drag_width > 0.0 {
                    ui.set_cursor_pos([title_right, 4.0]);
                    ui.invisible_button("##WindowDragRight", [drag_width, 28.0]);
                    handle_drag(ui, ctx);
                }

                let _spacing = ui.push_style_var(StyleVar::ItemSpacing([0.0, 0.0]));
                ui.set_cursor_pos([buttons_start, 4.0]);
                window_button(ui, ICON_MINIMIZE, false, false, || {
                    *ctx.window_action = Some(WindowAction::Minimize);
                });

                ui.set_cursor_pos([buttons_start + WINDOW_BUTTON_SIZE[0], 4.0]);
                window_button(
                    ui,
                    ICON_MAXIMIZE,
                    false,
                    ctx.maximize_hovered,
                    || {
                        *ctx.window_action = Some(WindowAction::ToggleMaximize);
                    },
                );

                ui.set_cursor_pos([buttons_start + WINDOW_BUTTON_SIZE[0] * 2.0, 4.0]);
                window_button(ui, ICON_CLOSE, true, false, || {
                    ctx.commands.send(EditorCommand::Exit);
                });
}

fn handle_drag(ui: &Ui, ctx: &mut UiContext) {
    if ui.is_item_hovered() {
        ui.set_mouse_cursor(Some(MouseCursor::Arrow));
    }

    if ui.is_item_hovered() && ui.is_mouse_double_clicked(MouseButton::Left) {
        *ctx.window_action = Some(WindowAction::ToggleMaximize);
    } else if ui.is_item_active() && ui.is_mouse_dragging(MouseButton::Left) {
        *ctx.window_action = Some(WindowAction::Drag);
    }
}

fn window_button(
    ui: &Ui,
    label: &str,
    close: bool,
    native_hovered: bool,
    action: impl FnOnce(),
) {
    let hovered = if close {
        [0.78, 0.17, 0.17, 1.0]
    } else {
        [0.20, 0.22, 0.25, 1.0]
    };
    let button_color = if native_hovered {
        hovered
    } else {
        [0.0, 0.0, 0.0, 0.0]
    };
    let button = ui.push_style_color(StyleColor::Button, button_color);
    let hover = ui.push_style_color(StyleColor::ButtonHovered, hovered);
    let active = ui.push_style_color(StyleColor::ButtonActive, hovered);

    if ui.button_with_size(label, WINDOW_BUTTON_SIZE) {
        action();
    }

    active.pop();
    hover.pop();
    button.pop();
}

