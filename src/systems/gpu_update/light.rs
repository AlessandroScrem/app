use crate::{
    LightComponent,
    renderer::
        gpu_manager::GpuManager
    ,
};

use legion::*;

#[system(for_each)]
#[filter(maybe_changed::<LightComponent>())]
pub fn update_light_uniform_to_gpu(
    light: &LightComponent,
    #[resource] queue: &wgpu::Queue,
    #[resource] gpu_manager: &GpuManager,
) {
    queue.write_buffer(
        &gpu_manager.light_uniform_buffer,
        0,
        bytemuck::bytes_of(&light.data),
    );
}
