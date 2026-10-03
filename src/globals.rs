#[derive(Clone, Debug)]
pub struct Globals {
    pub mips_cp: bool,
    pub light_enable: bool,
    pub ibl_enable: bool,
    pub skybox_enable: bool,
    pub skybox_enable_blur: bool,
    pub env_rotation: f32,
    pub exposure: f32,
    pub ibl_intensity: f32,
    pub tonemap_filter: u32,
    pub axis_enable: bool,
    pub bbox_enable: bool,
    pub bbox_axis_aligned: bool,
    pub debug_code: u32,
}

impl Default for Globals {
    fn default() -> Self {
        Self {
            mips_cp: false,
            light_enable: true,
            ibl_enable: true,
            skybox_enable: true,
            skybox_enable_blur: true,
            exposure: 1.0,
            env_rotation: 0.0,
            ibl_intensity: 1.0,
            tonemap_filter: 0,
            axis_enable: true,
            bbox_enable: false,
            bbox_axis_aligned: false,
            debug_code: 0,
        }
    }
}
