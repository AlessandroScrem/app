use crate::input::MouseButton;
use legion::*;

#[system]
pub fn camera_orbit(
    #[resource] camera: &mut crate::camera::Camera,
    #[resource] input: &crate::input::Input,
    #[resource] surface_configuration: &wgpu::SurfaceConfiguration,
) {
    let aspect =
        surface_configuration.width.max(1) as f32 / surface_configuration.height.max(1) as f32;
    camera.set_aspect(aspect);

    if input.is_mouse_button_down(MouseButton::Left) {
        let delta = (input.mouse_delta.x as f64, input.mouse_delta.y as f64);
        camera.orbit(delta);
    }

    if input.is_mouse_button_down(MouseButton::Middle) {
        let delta = (input.mouse_delta.x as f64, input.mouse_delta.y as f64);
        camera.pan(delta);
    }

    if let Some(delta) = input.mouse_wheel_movement {
        camera.zoom(delta.y);
    }
}

use crate::entities::bounding_box::BoundingBox;
use crate::{BoundingBoxComponent, GlobalModelComponent};
use legion::world::SubWorld;
use log::info;
#[system]
#[read_component(BoundingBoxComponent)]
#[read_component(GlobalModelComponent)]
pub fn recenter_camera(
    #[resource] camera: &mut crate::camera::Camera,
    #[resource] pick_object: &mut crate::picking::PickObject,
    world: &mut SubWorld,
) {
    if camera.recenter_request {
        camera.recenter_request = false;

        let bbox = {
            if let Some(selected) = pick_object.selected {
                get_bbox_from_entity(world, selected)
            } else {
                get_bounding_box_from_world(world)
            }
        };
        info!("Recenter Camera with box {:?}", bbox);
        crate::camera::center_camera_to_bounding_box(camera, bbox);
    }
}

fn get_bbox_from_entity(world: &mut SubWorld, entity: Entity) -> BoundingBox {
    if let Ok(entry) = world.entry_mut(entity) {
        let bounding_box = entry.get_component::<BoundingBoxComponent>().unwrap();
        bounding_box.global_bounding_box.clone()
    } else {
        BoundingBox::new_empty()
    }
}

fn get_bounding_box_from_world(world: &mut SubWorld) -> BoundingBox {
    let mut bbox = BoundingBox::new_empty();
    let mut query = <(&BoundingBoxComponent, &GlobalModelComponent)>::query();

    // FIXME: trasform local bbox with global matrix.. needs hierarchy update
    for (b, g) in query.iter(world) {
        bbox.merge(&b.bounding_box.transform_aabb(&g.mat));
    }

    bbox
}
