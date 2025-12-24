mod camera_update;
mod gpu_update;
mod gpu_render;
mod hierarchy;
mod picking;
mod registry_update;

use legion::Schedule;

// execution order on RedrawRequested:
// 1) current_scene_system
// 2) render_schedule_builder()
pub fn create_current_scene_schedule_builder() -> Schedule {
    Schedule::builder()
    .add_system(camera_update::camera_orbit_system())
    .add_system(picking::picking_system())
    .add_system(hierarchy::update_hieararchy_system()) 
    .flush()
    .add_system(camera_update::recenter_camera_system())
    .build()
}

pub fn create_render_schedule_builder() -> Schedule {
    Schedule::builder()
    .add_system(gpu_update::execute_start_system()) // create frame view and encoder
    .flush()
    .add_system(gpu_update::update_global_uniform_to_gpu_system())
    .add_system(gpu_update::update_light_uniform_to_gpu_system())
    .add_system(gpu_update::update_bounding_box_to_gpu_system()) // require hierarchy update
    .add_system(gpu_update::update_model_uniforms_to_gpu_system())
    .add_system(gpu_update::update_material_system_to_gpu_system())
    // render passes
    .add_system(gpu_render::render_mesh_system())
    .add_system(gpu_render::render_light_system())
    .add_system(gpu_render::render_skybox_system())
    .add_system(gpu_render::render_axis_system())
    .add_system(gpu_render::render_bounding_box_system())
    .add_system(gpu_render::render_hdr_to_ldr_system())
    .add_system(gpu_render::render_outline_system())
    .add_system(gpu_render::read_entity_id_to_buffer_system())
    .add_thread_local(gpu_render::render_imgui_system())

    // final update
    .add_system(gpu_update::execute_finish_system()) // submit encoder and present frame
    .build()
}

// execution order on about_to_wait():
// 1) update_schedule_builder (every 1 sec)
pub fn create_update_schedule_builder() -> Schedule {
    Schedule::builder()
        .add_system(registry_update::registry_update_system())
        .build()
}


