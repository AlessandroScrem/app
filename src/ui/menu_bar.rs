use super::ui_layer::{Layer, UiContext};
use crate::editor::EditorCommand;
use imgui::Ui;
use std::path::PathBuf;

#[derive(Default)]
pub struct MenuBarUi;
impl Layer for MenuBarUi {
    fn build(&mut self, ui: &Ui, ctx: &mut UiContext) {
        // let recent_files = &ctx.settings;

        if let Some(_bar) = ui.begin_main_menu_bar() {
            if let Some(_menu) = ui.begin_menu("File") {
                if ui.menu_item("New") {
                    ctx.connection.commands.send(EditorCommand::ClearScene);
                }
                if ui.menu_item("Open Scene") {
                    if let Some(path) = file_open(FileFilter::Json) {
                        ctx.connection
                            .commands
                            .send(EditorCommand::OpenScene { path });
                    }
                }
                if ui.menu_item("Save As..") {
                    if let Some(path) = file_save(FileFilter::Json) {
                        ctx.connection
                            .commands
                            .send(EditorCommand::SaveSceneAs { path });
                    }
                }
                if ui.menu_item("Save") {
                    ctx.connection.commands.send(EditorCommand::SaveScene);
                }
                ui.separator();
                if ui.menu_item("Load Gltf") {
                    if let Some(path) = file_open(FileFilter::Gltf) {
                        ctx.connection
                            .commands
                            .send(EditorCommand::LoadGltf { path });
                    }
                }
                if ui.menu_item("Add Ibl") {
                    if let Some(path) = file_open(FileFilter::Hdr) {
                        ctx.connection.commands.send(EditorCommand::AddIbl { path });
                    }
                }
                if ui.menu_item("Clear Scene") {
                    ctx.connection.commands.send(EditorCommand::ClearScene);
                }
                ui.separator();
                if ui.menu_item("Exit") {
                    ctx.connection.commands.send(EditorCommand::Exit);
                }
                ui.separator();
                ui.menu("Recent Files", || {
                    for (name, path) in ctx.scene_settings.recent.iter() {
                        if ui.menu_item(&name) {
                            ctx.connection
                                .commands
                                .send(EditorCommand::OpenScene { path: path.into() });
                        }
                    }
                    if ctx.scene_settings.recent.is_empty() {
                        ui.text_disabled("No recent files");
                    }
                });
            }
            if let Some(_menu) = ui.begin_menu("Edit") {
                ui.menu_item("Undo");
                ui.menu_item("Redo");
            }
            if let Some(_menu) = ui.begin_menu("View") {
                ui.menu_item("Show Stats");
            }
        }
    }
}

pub enum FileFilter {
    Gltf,
    Json,
    Hdr,
}
impl FileFilter {
    fn as_args(&self) -> (&str, &[&str]) {
        match self {
            Self::Gltf => ("glTF", &["gltf", "glb"]),
            Self::Json => ("json", &["json"]),
            Self::Hdr => ("hdr", &["hdr"]),
        }
    }
}
pub fn file_save(filter: FileFilter) -> Option<PathBuf> {
    let (name, ext) = filter.as_args();
    rfd::FileDialog::new().add_filter(name, ext).save_file()
}
pub fn file_open(filter: FileFilter) -> Option<PathBuf> {
    let (name, ext) = filter.as_args();
    rfd::FileDialog::new().add_filter(name, ext).pick_file()
}
