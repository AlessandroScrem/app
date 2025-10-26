mod axis;
mod bounding_box;
mod camera_orbit;
mod excute;
mod globals;
mod hdr;
mod hierarchy;
mod imgui;
mod light;
mod mesh;
mod outline;
mod picking;
mod registry_update;
mod skybox;

use legion::Schedule;

// execution order on RedrawRequested:
// 1) current_scene_system
// 2) render_schedule_builder()
pub fn create_current_scene_schedule_builder() -> Schedule {
    Schedule::builder()
    .add_system(camera_orbit::camera_orbit_system())
    .add_system(picking::picking_system())
    .build()
}

pub fn create_render_schedule_builder() -> Schedule {
    Schedule::builder()
    .add_system(excute::execute_start_system()) // create frame view and encoder
    .flush()
    .add_system(globals::update_global_uniform_to_gpu_system())
    .add_system(light::update_light_uniform_to_gpu_system())
    .add_system(hierarchy::hieararchy_system()) 
    .add_system(mesh::update_model_uniforms_to_gpu_system())
    .add_system(mesh::update_material_system_to_gpu_system())
    .add_system(bounding_box::update_bounding_box_to_gpu_system())
    .add_system(recenter_camera_system())
    // render passes
    .add_system(mesh::render_mesh_system())
    .add_system(light::render_light_system())
    .add_system(skybox::render_skybox_system())
    .add_system(axis::render_axis_system())
    .add_system(bounding_box::render_bounding_box_system())
    .add_system(hdr::render_hdr_to_ldr_system())
    .add_system(outline::render_outline_system())
    .add_system(picking::read_entity_id_to_buffer_system())
    .add_thread_local(imgui::render_imgui_system())
    .add_system(excute::execute_finish_system()) // submit encoder and present frame
    .build()
}

// execution order on about_to_wait():
// 1) update_schedule_builder (every 1 sec)
pub fn create_update_schedule_builder() -> Schedule {
    Schedule::builder()
        .add_system(registry_update::registry_update_system())
        .build()
}

use legion::*;
use legion::world::SubWorld;
use log::warn;
use crate::entities::bounding_box::BoundingBox;
use crate::BoundingBoxComponent;
#[system]
#[read_component(BoundingBoxComponent)]
pub fn recenter_camera(
    #[resource] camera: &mut crate::camera::Camera,
    world: &mut SubWorld,
) {
    if camera.recenter_request {
        camera.recenter_request = false;
        warn!("Recenter Camera");

        let bbox = get_bounding_box_from_world(world);
        crate::camera::center_camera_to_bounding_box(camera, bbox);
    }
}

pub fn get_bounding_box_from_world(world: &mut SubWorld) -> BoundingBox {
    let mut bbox = BoundingBox::new_empty();
    let mut query = <&BoundingBoxComponent>::query();

    for b in query.iter(world) {
        bbox.merge(&b.global_bounding_box);
    }

    bbox
}


