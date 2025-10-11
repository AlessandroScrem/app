pub mod globals;
pub mod mesh;
pub mod bounding_box;
pub mod axis;
pub mod camera_orbit;
pub mod imgui;
pub mod light;
pub mod skybox;
pub mod hdr;
pub mod picking;
pub mod outline;

use legion::Schedule;
pub fn create_render_schedule_builder() -> Schedule {
    Schedule::builder()
        .add_system(crate::systems::globals::global_system())
        .add_system(crate::systems::light::update_transform_system())
        .add_system(crate::systems::mesh::update_model_matrix_system())
        .add_system(crate::systems::bounding_box::update_bounding_box_system())
        .add_system(crate::systems::mesh::update_material_system())
        .add_system(crate::systems::mesh::mesh_system())
        .add_system(crate::systems::light::light_system())
        .add_system(crate::systems::skybox::skybox_system())
        .add_system(crate::systems::axis::axis_system())
        .add_system(crate::systems::bounding_box::bounding_box_system())
        .add_system(crate::systems::hdr::hdr_system())
        .add_system(crate::systems::outline::outline_system())
        .add_system(crate::systems::picking::read_entity_id_system())
        .add_thread_local(crate::systems::imgui::imgui_system())
        .build()
}