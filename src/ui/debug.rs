use super::{Layer, UiContext, UiTexture};
use imgui::*;

pub struct DebugUi {}

impl Layer for DebugUi {
    fn build(&mut self, ui: &Ui, ctx: &mut UiContext) {
        let debug_texture_id = ctx.snapshot.debug_texture_id;
        let texture_resolver = ctx.snapshot.texture_resolver;

        if let Some(id) = debug_texture_id {
            if let Some(id) = texture_resolver.resolve(UiTexture::Engine(id)) {
                let window = ui.window("Debug Texture");
                let name = "No Name";
                window
                    .size([256.0, 256.0], Condition::FirstUseEver)
                    .position([400.0, 0.0], Condition::FirstUseEver)
                    .build(|| {
                        ui.image_button(name, id, [200.0, 200.0]);
                        ui.same_line();
                        ui.text(name);
                        ui.separator();
                    });
            }
        }
    }
}
