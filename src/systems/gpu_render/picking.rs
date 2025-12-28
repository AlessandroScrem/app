use crate::{input::Input, picking::PickObject, renderer::GpuManager};
use legion::*;

#[system]
pub fn read_entity_id_to_buffer(
    #[resource] encoder: &mut wgpu::CommandEncoder,
    #[resource] gpu_manager: &GpuManager,
    #[resource] pick_object: &mut PickObject,
    #[resource] input: &Input,
) {
    if  input.is_cursor_moved() && pick_object.buffer.ready(){

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
                buffer: &pick_object.buffer.buffer,
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
