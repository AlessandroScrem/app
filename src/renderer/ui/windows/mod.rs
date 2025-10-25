pub mod entities;
pub mod properties;
pub mod settings;

use super::*;
pub use settings::draw_window_settings;
pub use entities::draw_window_entities;
pub use properties::draw_window_properties;

pub fn draw_demo_window(ctx: &InspectorContext) {
    if *ctx.demo_open {
        ctx.ui.show_demo_window(&mut true);
    }
}

pub fn draw_debug_texture(ctx: &InspectorContext) {
    let registry = ctx.resources.get::<ImGuiTextureRegistry>().unwrap();
    let ui = ctx.ui;

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
