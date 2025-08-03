use std::sync::Arc;

use crate::camera::Camera;
use crate::renderer::uniform::CameraUniform;
use crate::resources::gpu_manager::GPUResourceManager;

pub fn create() -> impl legion::systems::Runnable {
    use legion::IntoQuery;
    use legion::{Read, SystemBuilder};

    SystemBuilder::new("update globals")
        .read_resource::<Arc<GPUResourceManager>>()
        .write_resource::<wgpu::Queue>()
        .with_query(<(Read<Camera>,)>::query())
        .build(
            |_, world, (resource_manager, queue), camera_query| {

                let filtered_camera_data: Vec<_> = camera_query.iter(world).collect();
                let camera = filtered_camera_data.first();

                if camera.is_none() {
                    return;
                }
                let camera = &camera.as_ref().unwrap().0;

                update_globals(
                    camera,
                    queue,
                    resource_manager,
                );
            },
        )
}

pub fn update_globals<'a>(
    camera: &crate::camera::Camera,
    queue:  &wgpu::Queue,
    resource_manager: &GPUResourceManager,
)  {

    let updated_uniforms =  CameraUniform {
        view_position: camera.get_position().to_homogeneous().into(),
        view_proj: (camera.get_projection() * camera.get_matrix()).into(),
    };

    queue.write_buffer(
        &resource_manager.camera_uniform_buffer,
        0,
        bytemuck::bytes_of(&updated_uniforms)
    );
}
