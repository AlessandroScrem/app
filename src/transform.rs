pub struct Transform {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
} 

use cgmath::{Matrix4, Quaternion};
use cgmath::{Euler, Rad, Vector3};
impl Transform {
    pub fn compute_model_matrix(&self) -> Matrix4<f32> {
        // converte la rotazione xyz "Radianti" in Quaternion
        fn to_quat(r: &[f32; 3]) -> Quaternion<f32> {
            let euler = Euler::new(Rad(r[0]), Rad(r[1]), Rad(r[2]));
            Quaternion::from(euler)
        }

        let translation = Vector3::from(self.position);
        let rotation = to_quat(&self.rotation);
        let scale = Vector3::from(self.scale);

        let t = Matrix4::from_translation(translation);
        let r = Matrix4::from(rotation);
        let s = Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z);

        t * r * s
    }
}
