pub mod entities;
pub mod registry;
pub mod settings;
pub mod imgui_layer;
pub mod tools;
pub mod properties;

pub use registry::ImGuiTextureRegistry;
pub use imgui_layer::ImguiLayer;
pub use imgui_layer::UiContext;

pub use imgui::*;

pub use entities::*;
pub use settings::*;
pub use properties::*;

pub fn draw_debug_texture(ui: &imgui::Ui, ctx: &UiContext) {
    let registry = ctx.registry;

    let debug_tex_path = std::path::Path::new("debug_texture");
    let name = debug_tex_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("no name");

    if let Some(id) = registry.ids.get(debug_tex_path) {
        let window = ui.window("Debug Texture");
        window
            .size([256.0, 256.0], Condition::FirstUseEver)
            .position([400.0, 0.0], Condition::FirstUseEver)
            .build(|| {
                ui.image_button(name, *id, [200.0, 200.0]);
                ui.same_line();
                ui.text(name);
                ui.separator();
            });
    }
}
