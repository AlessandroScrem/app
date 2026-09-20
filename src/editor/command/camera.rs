#[derive(Clone, Debug)]
pub enum CameraCommand {
    Recenter,
    SetFov(f32),
    SetDistance(f32),
    SetNearFar { near: f32, far: f32 },
}
