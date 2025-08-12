
use crate::transform::Transform;
use legion::*;

/// A function to help create a camera entity.
pub fn create(world: &mut World, transform: Transform) -> Entity {
     world.push((transform, ))

}


