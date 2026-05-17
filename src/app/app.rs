use crate::prelude::*;

use crate::scene::Scene;

use legion::Entity;
use legion::Resources;

#[derive(Default)]
pub struct App {
    pub current_scene: Scene,
    pub asset_mgr: AssetManager,
    pub resources: Resources,
    pub globals: Globals,
    pub camera: Camera,
    pub domain_events: DomainEvents,
    pub selected: Option<Entity>,
    pub hovered: Option<Entity>,
    pub exit_requested: bool,
}

impl App {
    pub fn handle_selection_input(&mut self, input: &crate::input::Input) {
        use crate::input::MouseButton;
        use winit::keyboard::{Key, NamedKey};
        if input.is_mouse_button_pressed(MouseButton::Left)
            && input.is_key_down(Key::Named(NamedKey::Alt))
        {
            self.selected = self.hovered;
        }
    }

    pub fn update_scene(&mut self) {
        self.current_scene
            .schedule
            .execute(&mut self.current_scene.world, &mut self.resources);
    }
}
