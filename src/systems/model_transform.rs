use crate::renderer::uniform::ModelUniform;
use crate::resources::gpu_manager::GPUResourceManager;
use crate::transform::Transform;

use cgmath::{Vector3};
use legion::*;
use std::sync::Arc;

#[system(for_each)]
pub fn nodel_transform(
    transform: &Transform,
    #[resource] resource_manager: &Arc<GPUResourceManager>,
    #[resource] queue: &wgpu::Queue,
) {
    update_trnsform(transform, queue, resource_manager);
    
}

pub fn update_trnsform(transform: &Transform, queue: &wgpu::Queue, resource_manager: &GPUResourceManager) {    
    let updated_uniforms = ModelUniform {
        model: cgmath::Matrix4::from_translation(Vector3::from(transform.position)).into(),
    };
    
    queue.write_buffer(
        &resource_manager.model_uniform_buffer,
        0,
        bytemuck::bytes_of(&updated_uniforms),
    );
}

/* 
use cgmath::{BaseFloat, Matrix3, Matrix4, Quaternion};
fn compute_model_matrix<T: BaseFloat>(translation: Vector3<T>, rotation: Quaternion<T>, scale: Vector3<T>) -> Matrix4<T> {
    let t = Matrix4::from_translation(translation);
    let r = Matrix4::from(rotation);
    let s = Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z);

    t * r * s
}

// Per la normal matrix puoi usare solo la matrice di rotazione:
fn compute_normal_matrix<T: BaseFloat>(rotation: Quaternion<T>) -> Matrix3<T> {
    Matrix3::from(rotation)
}
 */