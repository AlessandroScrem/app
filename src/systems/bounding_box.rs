use crate::entities::components::{BoundingBoxComponent, GlobalModelComponent};
use legion::*;

#[system(for_each)]
#[filter(maybe_changed::<GlobalModelComponent>())]
pub fn update_bounding_box(
    global_model: &GlobalModelComponent,
    bbox_component: &mut BoundingBoxComponent,
) {
    bbox_component.global_bounding_box = bbox_component
        .bounding_box
        .transform_aabb(&global_model.mat);
}
