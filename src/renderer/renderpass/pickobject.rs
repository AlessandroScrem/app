use crate::{input::Input, renderer::gpu_renderer::GpuView};

pub struct PickObjectRenderPass<'a> {
    gpu: GpuView<'a>,
    encoder: &'a mut wgpu::CommandEncoder,
}

impl<'a> PickObjectRenderPass<'a> {
    pub fn new(gpu: GpuView<'a>, encoder: &'a mut wgpu::CommandEncoder) -> Self {
        Self { gpu, encoder }
    }

    pub fn render(self, input: &Input) {
        let pickobject = self.gpu.pickobject;
        let gpu_manager = self.gpu.gpu_mgr;
        let encoder = self.encoder;

        if input.is_cursor_moved() && !pickobject.pending {
            let aligned_bytes_per_row = 256; // minimo richiesto
            let size = gpu_manager.entity_id_texture._texture.size();
            let mouse_pos_x = input.mouse_position.x as u32;
            let mouse_pos_y = input.mouse_position.y as u32;
            let x = mouse_pos_x.clamp(0, size.width - 1);
            let y = mouse_pos_y.clamp(0, size.height - 1);

            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture: &gpu_manager.entity_id_texture._texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x, y, z: 0 },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &pickobject.buffer,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(aligned_bytes_per_row),
                        rows_per_image: Some(1),
                    },
                },
                wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
            );
        }
    }
}
