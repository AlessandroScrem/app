
use crate::camera::Camera;
use legion::*;

/// A function to help create a camera entity.
pub fn create(world: &mut World, camera: Camera) -> Entity {
     world.push((camera, ))

}


