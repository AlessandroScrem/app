use crate::{
    BoundingBoxComponent, GlobalModelComponent, Globals, renderer::bbox_manager,
};
use legion::*;

#[system(for_each)]
#[filter(maybe_changed::<GlobalModelComponent>())]
pub fn update_bounding_box_to_gpu(
    global_model: &GlobalModelComponent,
    bbox_component: &mut BoundingBoxComponent,
    entity: &Entity,
    #[resource] device: &wgpu::Device,
    #[resource] queue: &wgpu::Queue,
    #[resource] globals: &Globals,
    #[resource] bbox_manager: &mut bbox_manager::BBoxManager,
) {
    
    let vertices = {
        if globals.bbox_axis_aligned {
            bbox_component.gen_aabb_vertices()
        } else {
            bbox_component.gen_obb_vertices(&global_model.mat)
        }
    };

    queue.write_buffer(
        &bbox_manager.get_or_create(device, *entity),
        0,
        bytemuck::cast_slice(&vertices.as_slice()),
    );
}

