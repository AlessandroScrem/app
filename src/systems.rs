pub mod axis;
pub mod bounding_box;
pub mod camera_orbit;
pub mod excute;
pub mod globals;
pub mod hdr;
pub mod hierarchy;
pub mod imgui;
pub mod light;
pub mod mesh;
pub mod outline;
pub mod picking;
pub mod registry_update;
pub mod skybox;

use legion::Schedule;

// execution order on RedrawRequested:
// 1) current_scene_system
// 2) render_schedule_builder()
pub fn create_current_scene_schedule_builder() -> Schedule {
    Schedule::builder()
    .add_system(crate::systems::camera_orbit::camera_orbit_system())
    .add_system(crate::systems::picking::picking_system())
    .build()
}

pub fn create_render_schedule_builder() -> Schedule {
    Schedule::builder()
    .add_system(crate::systems::excute::execute_start_system()) // create frame view and encoder
    .flush()
    .add_system(crate::systems::globals::update_global_uniform_to_gpu_system())
    .add_system(crate::systems::light::update_light_uniform_to_gpu_system())
    .add_system(crate::systems::hierarchy::hieararchy_system()) 
    .add_system(crate::systems::mesh::update_model_uniforms_to_gpu_system())
    .add_system(crate::systems::mesh::update_material_system_to_gpu_system())
    .add_system(crate::systems::bounding_box::update_bounding_box_to_gpu_system())
    // render passes
    .add_system(crate::systems::mesh::render_mesh_system())
    .add_system(crate::systems::light::render_light_system())
    .add_system(crate::systems::skybox::render_skybox_system())
    .add_system(crate::systems::axis::render_axis_system())
    .add_system(crate::systems::bounding_box::render_bounding_box_system())
    .add_system(crate::systems::hdr::render_hdr_to_ldr_system())
    .add_system(crate::systems::outline::render_outline_system())
    .add_system(crate::systems::picking::read_entity_id_to_buffer_system())
    .add_thread_local(crate::systems::imgui::render_imgui_system())
    .add_system(crate::systems::excute::execute_finish_system()) // submit encoder and present frame
    .build()
}

// execution order on about_to_wait():
// 1) update_schedule_builder (every 1 sec)
pub fn create_update_schedule_builder() -> Schedule {
    Schedule::builder()
        .add_system(crate::systems::registry_update::registry_update_system())
        .build()
}
