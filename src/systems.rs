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
pub fn create_render_schedule_builder() -> Schedule {
    Schedule::builder()
        .add_system(crate::systems::excute::execute_start_system()) // create frame view and encoder
        .flush()
        .add_system(crate::systems::globals::global_system())
        .add_system(crate::systems::light::update_transform_system())
        .add_system(crate::systems::mesh::update_model_matrix_system())
        .add_system(crate::systems::hierarchy::hieararchy_system()) // hierarchy
        .add_system(crate::systems::hierarchy::hierarchy_update_uniforms_system()) // hierarchy
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
        .add_system(crate::systems::excute::execute_finish_system()) // submit encoder and present frame
        .build()
}

pub fn create_update_schedule_builder() -> Schedule {
    Schedule::builder()
        .add_system(crate::systems::registry_update::registry_update_system())
        .build()
}

pub fn create_current_scene_schedule_builder() -> Schedule {
    Schedule::builder()
        .add_system(crate::systems::camera_orbit::camera_orbit_system())
        .add_system(crate::systems::picking::picking_system())
        .build()
}
