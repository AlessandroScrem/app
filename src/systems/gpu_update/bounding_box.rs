use crate::{
    BoundingBoxComponent, GlobalModelComponent, Globals,
};
use legion::*;

#[system(for_each)]
#[filter(maybe_changed::<GlobalModelComponent>())]
pub fn update_bounding_box_to_gpu(
    global_model: &GlobalModelComponent,
    bbox_component: &mut BoundingBoxComponent,
    #[resource] queue: &wgpu::Queue,
    #[resource] globals: &Globals,
) {

    bbox_component.global_bounding_box = bbox_component
        .bounding_box
        .transform_aabb(&global_model.mat);

    let vertices = {
        if globals.bbox_axis_aligned {
            bbox_component.gen_aabb_vertices()
        } else {
            bbox_component.gen_obb_vertices(&global_model.mat)
        }
    };

    queue.write_buffer(
        &bbox_component.vertex_buffer,
        0,
        bytemuck::cast_slice(&vertices.as_slice()),
    );
}

