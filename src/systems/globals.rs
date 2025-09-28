use crate::camera::Camera;
use crate::renderer::gpu_manager::GPUResourceManager;
use crate::renderer::uniform::CameraUniform;
use crate::{GlobalUniform, Globals};

use legion::*;
use std::sync::Arc;

#[system]
pub fn global(
    #[resource] resource_manager: &Arc<GPUResourceManager>,
    #[resource] queue: &wgpu::Queue,
    #[resource] camera: &Camera,
    #[resource] globals: &Globals,
    #[resource] surface_config: &wgpu::SurfaceConfiguration,
) {
    let screen_size = [surface_config.width as f32, surface_config.height as f32];
    update_globals(
        camera,
        globals,
        queue,
        &resource_manager.camera_uniform_buffer,
        &resource_manager.globals_uniform_buffer,
        screen_size,
    );
}

pub fn update_globals(
    camera: &Camera,
    globals: &Globals,
    queue: &wgpu::Queue,
    camera_uniform_buffer: &wgpu::Buffer,
    globals_uniform_buffer: &wgpu::Buffer,
    screen_size: [f32; 2],
) {
    let updated_camera_uniform = CameraUniform {
        view_position: camera.get_position().to_homogeneous().into(),
        view: camera.get_view_mat().into(),
        proj: camera.get_projection_mat().into(),
        screen_size,
        ..Default::default()
    };

    let updated_globals_uniform = GlobalUniform {
        ibl_enable: globals.ibl_enable as u32,
        skybox_enable: globals.skybox_enable as u32,
        exposure: globals.exposure,
        tonemap_filter: globals.tonemap_filter,
    };

    queue.write_buffer(
        camera_uniform_buffer,
        0,
        bytemuck::bytes_of(&updated_camera_uniform),
    );
    queue.write_buffer(
        globals_uniform_buffer,
        0,
        bytemuck::bytes_of(&updated_globals_uniform),
    );
}
