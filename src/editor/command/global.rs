#[derive(Clone, Debug)]
pub enum GlobalCommand {
    SetLightEnable(bool),
    SetIblEnable(bool),
    SetSkyboxEnable(bool),
    SetSkyboxBlur(bool),
    SetAxisEnable(bool),
    SetBoundingBoxEnable(bool),
    SetBoundingBoxAxisAligned(bool),
    SetMipsWithCompute(bool),
    SetEnvironmentRotation(f32),
    SetDebugCode(u32),
    SetExposure(f32),
    SetIblIntensity(f32),
    SetTonemap(u32),
}
