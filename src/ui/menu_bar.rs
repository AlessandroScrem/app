use std::path::PathBuf;

use super::*;
use imgui::*;

pub struct MenuBarUi {}

const LANTERN: &str = "./assets/Lantern/Lantern.gltf";
const SPONZA: &str = "C:/Users/aless/Downloads/glTF-Sample-Models/2.0/Sponza/glTF/Sponza.gltf";
const TRANSMISSION_TEST: &str = "c:/Users/aless/Downloads/glTF-Sample-Assets/Models/TransmissionTest/glTF/TransmissionTest.gltf";

impl Layer for MenuBarUi {
    fn build(&mut self, ui: &Ui, ctx: &mut UiContext) {
        use AssetEvent::*;
        use DomainEvent::*;
        
        if let Some(_menu_bar) = ui.begin_main_menu_bar() {
            if let Some(_file_menu) = ui.begin_menu("File") {
                if ui.menu_item("New") {}
                if ui.menu_item("Open") {
                    menu_bar::file_open("gltf").map(|f| ctx.write.push(Assets(LoadGltf(f))));
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
                if ui.menu_item("Save") {}
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

pub fn file_open(filter: &str) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter(filter, &[filter])
        .pick_file()
}
