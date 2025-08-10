use crate::camera::Camera;
use crate::renderer::uniform::CameraUniform;
use crate::resources::gpu_manager::GPUResourceManager;

use std::sync::Arc;
use legion::*;

#[system(for_each)]
pub fn global(
    camera: &Camera,
    #[resource] resource_manager: &Arc<GPUResourceManager>,
    #[resource] queue: &wgpu::Queue,
) {
    update_globals(camera, queue, resource_manager);
}

pub fn update_globals<'a>(
    camera: &crate::camera::Camera,
    queue: &wgpu::Queue,
    resource_manager: &GPUResourceManager,
) {
    let updated_uniforms = CameraUniform {
        view_position: camera.get_position().to_homogeneous().into(),
        view_proj: (camera.get_projection() * camera.get_matrix()).into(),
    };

    queue.write_buffer(
        &resource_manager.camera_uniform_buffer,
        0,
        bytemuck::bytes_of(&updated_uniforms),
    );
}
