pub mod globals;
pub mod mesh;
pub mod camera_orbit;
pub mod imgui;
pub mod light;

use legion::Schedule;
pub fn create_render_schedule_builder() -> Schedule {
    Schedule::builder()
        .add_system(crate::systems::globals::global_system())
        .add_system(crate::systems::light::update_trnsform_system())
        .add_system(crate::systems::mesh::update_light_system())
        .add_system(crate::systems::mesh::mesh_system())
        .add_system(crate::systems::light::light_system())
        .add_thread_local(crate::systems::imgui::imgui_system())
        .build()
}