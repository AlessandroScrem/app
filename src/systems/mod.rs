
mod hierarchy;
mod bounding_box;

use legion::Schedule;

pub(crate) fn create_current_scene_schedule_builder() -> Schedule {
    Schedule::builder()
    .add_system(hierarchy::update_hieararchy_system()) 
    .flush()
    .add_system(bounding_box::update_bounding_box_system())
    .build()
}


