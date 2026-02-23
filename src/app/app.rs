use crate::prelude::*;

use crate::engine::RunningApp;
use crate::scene::Scene;

use legion::Entity;
use legion::Resources;

#[derive(Default)]
pub(crate) struct App {
    pub(crate) current_scene: Scene,
    pub(crate) asset_mgr: AssetManager,
    pub(crate) resources: Resources,
    pub(crate) globals: Globals,
    pub(crate) camera: Camera,
    pub(crate) domain_events: DomainEvents,
    pub(crate) selected: Option<Entity>,
    pub(crate) hovered: Option<Entity>,
}

impl App {
    pub(crate) fn update_selected(&mut self, runtime: &mut RunningApp) {
        let input = &runtime.input;
        let renderer = &mut runtime.renderer;
        // update hovered entity_id from buffer
        use crate::input::MouseButton;
        use winit::keyboard::{Key, NamedKey};
        if input.is_cursor_moved() {
            self.hovered = renderer.get_hovered();
        }

        if input.is_mouse_button_pressed(MouseButton::Left)
            && input.is_key_down(Key::Named(NamedKey::Alt))
        {
            self.selected = self.hovered;
        }
    }

    pub(crate) fn update_scene(&mut self) {
        self.current_scene
            .schedule
            .execute(&mut self.current_scene.world, &mut self.resources);
    }

}
