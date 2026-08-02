use crate::gpu::ReadbackState;

use super::*;

#[derive(Default)]
pub struct PickObjectPass {}

impl PickObjectPass {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RenderPass for PickObjectPass {
    fn name(&self) -> &'static str {
        "PickObjectPass"
    }

    fn reads(&self) -> &[ResourceId] {
        &[ResourceId::ENTITY]
    }
    fn writes(&self) -> &[ResourceId] {
        &[ResourceId::PICKBUFFER]
    }

    fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        ctx: &mut RenderContext,
        _frame: &FrameData,
    ) {
        let pickobject = &mut ctx.pickobject;
        let gpu_manager = ctx.gpu_mgr;

        if !matches!(pickobject.state, ReadbackState::Idle) {
            return;
        }

        let size = gpu_manager
            .get_framebuffer_texture(FramebufferKind::EntityId)
            .size();

        let (mouse_pos_x, mouse_pos_y ) = pickobject.get_picking_coords();
        let origin = wgpu::Origin3d {
            x: mouse_pos_x.clamp(0, size.width - 1),
            y: mouse_pos_y.clamp(0, size.height - 1),
            z: 0
        };
        const ALIGNED_BYTES_PER_ROW:u32 = 256; // minimo richiesto
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: gpu_manager.get_framebuffer_texture(FramebufferKind::EntityId),
                mip_level: 0,
                origin,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &pickobject.buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(ALIGNED_BYTES_PER_ROW),
                    rows_per_image: Some(1),
                },
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        pickobject.state = ReadbackState::CopySubmitted;
    }
}
