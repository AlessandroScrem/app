use crate::camera::Camera;
use crate::renderer::uniform::CameraUniform;
use crate::resources::gpu_manager::GPUResourceManager;

use std::sync::Arc;
use legion::*;

#[system]
pub fn global(
    #[resource] resource_manager: &Arc<GPUResourceManager>,
    #[resource] queue: &wgpu::Queue,
    #[resource] camera: &Camera,
    #[resource] surface_config:  &wgpu::SurfaceConfiguration,
) {
    let screen_size = [surface_config.width as f32, surface_config.height as f32];
    update_globals(camera, queue, &resource_manager.camera_uniform_buffer, screen_size);
}

pub fn update_globals(
    camera: &crate::camera::Camera,
    queue: &wgpu::Queue,
    camera_uniform_buffer: &wgpu::Buffer,
    screen_size: [f32; 2],
) {

    let updated_uniforms = CameraUniform {
        view_position: camera.get_position().to_homogeneous().into(),
        view: camera.get_view_mat().into(),
        proj: camera.get_projection_mat().into(),
        screen_size,
        ..Default::default()
    };

    queue.write_buffer(
        camera_uniform_buffer,
        0,
        bytemuck::bytes_of(&updated_uniforms),
    );
}
