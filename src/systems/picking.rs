use crate::{input::{Input, MouseButton}, picking::PickObject};
use legion::*;

#[system]
pub fn picking(
    #[resource] pick_object: &mut PickObject,
    #[resource] input: &Input,
) {
    
    // read hovered entity_id from buffer
    if input.is_cursor_moved() {
        pick_object.apply();
    }

    if input.is_mouse_button_pressed(MouseButton::Left) {
        pick_object.select_hovered();
    }
}

