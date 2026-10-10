use crate::app::domain::events::DomainEvent::Selection;
use crate::app::domain::events::SelectionEvent::Hovered;
use crate::editor::{EditorCommand, SelectionCommand};
use crate::engine::editor::EditorService;
use crate::engine::engine::EventBus;
use crate::engine::readback::{QueryResult, ReadbackManager};
use crate::gpu::context::GpuContextRef;
use crate::input::Input;
use crate::EntityRawU64;
use legion::Entity;

/// Owns asynchronous GPU readback and translates picking results into app/editor events.
#[derive(Default)]
pub(crate) struct PickingService {
    readback: ReadbackManager,
}

impl PickingService {
    pub(crate) fn handle_input(
        &mut self,
        input: &Input,
        gpu: &GpuContextRef,
        entity_id_texture: &wgpu::Texture,
        bus: &mut EventBus,
        editor_service: &EditorService,
    ) {
        if let Some(result) = self.readback.poll_results() {
            match result {
                QueryResult::Pick(id) => {
                    bus.send_domain(Selection(Hovered(id.map(Entity::from_raw_u64))));
                }
                QueryResult::Selection(entities) => {
                    editor_service.send_command(EditorCommand::Selection(
                        SelectionCommand::Select { entities },
                    ));
                }
            }
        }

        if input.is_cursor_moved() {
            self.request_pick(
                gpu,
                entity_id_texture,
                (input.mouse_position.x as u32, input.mouse_position.y as u32),
            );
        }
    }

    pub(crate) fn request_pick(
        &mut self,
        gpu: &GpuContextRef,
        entity_id_texture: &wgpu::Texture,
        position: (u32, u32),
    ) {
        self.readback
            .request_pick(gpu, entity_id_texture, position);
    }

    pub(crate) fn request_selection(
        &mut self,
        gpu: &GpuContextRef,
        entity_id_texture: &wgpu::Texture,
        origin: (u32, u32),
        size: (u32, u32),
    ) {
        self.readback
            .request_selection(gpu, entity_id_texture, origin, size);
    }
}
