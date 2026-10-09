#[derive(Clone, Debug)]
pub enum CameraCommand {
    Recenter,
    SetFov(f32),
    SetDistance(f32),
    SetNearFar { near: f32, far: f32 },
}

impl CameraCommand {
    pub fn settings_changed(&self) -> bool {
        matches!(
            self,
            Self::SetFov(_) | Self::SetDistance(_) | Self::SetNearFar { .. }
        )
    }
}
