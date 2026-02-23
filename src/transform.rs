use crate::math::*;
use crate::TransformComponent;

impl Default for TransformComponent {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
} 

impl TransformComponent {
    pub(crate) fn compute_model_matrix(&self) -> Mat4 {
        // converte la rotazione xyz "Radianti" in Quaternion
        fn to_quat(r: &[f32; 3]) -> Quat {
            let euler = Euler::new(Rad(r[0]), Rad(r[1]), Rad(r[2]));
            Quat::from(euler)
        }

        let translation = Vec3::from(self.position);
        let rotation = to_quat(&self.rotation);
        let scale = Vec3::from(self.scale);

        let t = Mat4::from_translation(translation);
        let r = Mat4::from(rotation);
        let s = Mat4::from_nonuniform_scale(scale.x, scale.y, scale.z);

        t * r * s
    }
}
