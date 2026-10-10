use crate::engine::readback::{QueryResult, ReadbackManager};
use crate::gpu::context::GpuContextRef;

/// Owns asynchronous GPU readback state for object picking and area selection.
#[derive(Default)]
pub(crate) struct PickingService {
    readback: ReadbackManager,
}

impl PickingService {
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

    pub(crate) fn poll_results(&mut self) -> Option<QueryResult> {
        self.readback.poll_results()
    }
}
