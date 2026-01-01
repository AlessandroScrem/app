use crate::{application_handler::WindowEventQueue, input::Input, prelude::ui::ImguiState};
use legion::*;
use winit::event::Event;

#[system]
pub fn input_update(
    #[resource] imgui: &mut ImguiState,
    #[resource] event_queue: &mut WindowEventQueue,
    #[resource] input: &mut Input,
) {
    while let Some(event) = event_queue.queue.pop_front() {
        imgui
            .platform
            .handle_event::<()>(imgui.context.io_mut(), &event_queue.window, &event);

        let io = imgui.context.io();

        match &event {
            Event::DeviceEvent { .. } | Event::WindowEvent { .. } => {
                if !io.want_capture_mouse {
                    input.update_events(&event);
                }
            }
            _ => {}
        }
    }
}
