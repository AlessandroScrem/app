use std::path::PathBuf;

use super::*;
use crate::{app::domain::events::{AssetEvent, DomainEvent, SceneEvent}, asset_path};
use imgui::*;

pub struct MenuBarUi {}

const LANTERN: &str = asset_path!("Lantern/Lantern.gltf");
const SPONZA: &str = "c:/Users/aless/Downloads/glTF-Sample-Assets/Models/Sponza/glTF/Sponza.gltf";
const TRANSMISSION_TEST: &str = "c:/Users/aless/Downloads/glTF-Sample-Assets/Models/TransmissionTest/glTF/TransmissionTest.gltf";
const DAMAGED_HELMET: &str =
    "c:/Users/aless/Downloads/glTF-Sample-Assets/Models/DamagedHelmet/glTF/DamagedHelmet.gltf";

impl Layer for MenuBarUi {
    fn build(&mut self, ui: &Ui, ctx: &mut UiContext) {
        use AssetEvent::*;
        use DomainEvent::*;
        use SceneEvent::*;

        if let Some(_menu_bar) = ui.begin_main_menu_bar() {
            if let Some(_file_menu) = ui.begin_menu("File") {
                if ui.menu_item("New") {}
                if ui.menu_item("Open Scene") {
                    menu_bar::file_open(FileFilter::Json).map(|f| ctx.write.push(Scene(Open(f))));
                }
                if ui.menu_item("Save Scene") {
                    menu_bar::file_save(FileFilter::Json).map(|f| ctx.write.push(Scene(Save(f))));
                }
                ui.separator();
                if ui.menu_item("Load Gltf") {
                    menu_bar::file_open(FileFilter::Gltf)
                        .map(|f| ctx.write.push(Assets(LoadGltf(f))));
                }
                if ui.menu_item("Clear Scene") {
                    ctx.write.push(Scene(ClearScene));
                }
                ui.separator();
                if ui.menu_item("Sponza") {
                    ctx.write.push(Assets(LoadGltf(SPONZA.into())));
                }
                if ui.menu_item("lantern") {
                    ctx.write.push(Assets(LoadGltf(LANTERN.into())));
                }
                if ui.menu_item("Transmission_Test") {
                    ctx.write.push(Assets(LoadGltf(TRANSMISSION_TEST.into())));
                }
                if ui.menu_item("Damaged Helmet") {
                    ctx.write.push(Assets(LoadGltf(DAMAGED_HELMET.into())));
                }
                if ui.menu_item("Exit") {
                    ctx.write.push(Exit);
                }
            }

            if let Some(_edit_menu) = ui.begin_menu("Edit") {
                if ui.menu_item("Undo") {}
                if ui.menu_item("Redo") {}
            }

            if let Some(_view_menu) = ui.begin_menu("View") {
                if ui.menu_item("Show Stats") {}
            }
        }
    }
}

pub enum FileFilter {
    Gltf,
    Json,
}

impl FileFilter {
    fn as_args(&self) -> (&str, &[&str]) {
        match self {
            FileFilter::Gltf => ("glTF", &["gltf", "glb"]),
            FileFilter::Json => ("json", &["json"]),
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
