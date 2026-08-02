#![allow(unused_imports)]
use cgmath::*;
pub type Mat3 = Matrix3<f32>;
pub type Mat4 = Matrix4<f32>;
pub type Vec2 = Vector2<f32>;
pub type Vec3 = Vector3<f32>;
pub type Vec4 = Vector4<f32>;
pub type Point3f = Point3<f32>;
pub type Quat = Quaternion<f32>;
pub use cgmath::{
    Angle, Array, Deg, EuclideanSpace, Euler, InnerSpace as _, Matrix as _, One, Rad,
    Rotation3 as _, SquareMatrix as _, Transform as _, Zero,
    num_traits::{one, zero},
    vec3, vec4,
};

// cgmath matrix is RH OpenGL-style (Z NDC is [-1 e 1])
// (OPENGL_TO_WGPU_MATRIX) will correct Z NDC from opengl [-1, 1] to Vulkan(wgpu) Z [0, 1]
// TODO: implement projection LH with Z [0, 1]
#[rustfmt::skip]
    pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.0,
        0.0, 0.0, 0.5, 1.0,
    );
pub fn perspective<A: Into<Rad<f32>>>(
    fovy: A,
    aspect: f32,
    near: f32,
    far: f32,
) -> cgmath::Matrix4<f32> {
    OPENGL_TO_WGPU_MATRIX * cgmath::perspective(fovy, aspect, near, far)
}

pub fn ortho(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> cgmath::Matrix4<f32> {
    OPENGL_TO_WGPU_MATRIX * cgmath::ortho(left, right, bottom, top, near, far)
}

// pub fn vec3_min(a: &Vec3, b: &Vec3) -> Vec3 {
//     Vec3 {
//         x: a.x.min(b.x),
//         y: a.y.min(b.y),
//         z: a.z.min(b.z),
//     }
// }

// pub fn vec3_max(a: &Vec3, b: &Vec3) -> Vec3 {
//     Vec3 {
//         x: a.x.max(b.x),
//         y: a.y.max(b.y),
//         z: a.z.max(b.z),
//     }
// }
