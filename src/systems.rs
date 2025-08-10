pub mod globals;
pub mod mesh;
pub mod camera_orbit;
pub mod imgui;

use legion::Schedule;
pub fn create_render_schedule_builder() -> Schedule {
    Schedule::builder()
        .add_system(crate::systems::globals::create())
        .add_system(crate::systems::mesh::create())
        .add_thread_local(crate::systems::imgui::create())
        .build()
}