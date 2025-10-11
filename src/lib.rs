mod app;
mod application_handler;
mod camera;
mod renderer;
mod scene;
pub mod assets;
pub mod entities;
pub mod input;
pub mod systems;
pub mod transform;
pub mod test_utils;
pub mod picking;
pub mod timestep;


pub mod prelude {
    pub use log::{info, debug, warn, error};
    pub use crate::timestep;
    pub use super::app::App;
    pub use crate::camera::Camera;
    pub use crate::renderer::Renderer;
    pub use crate::renderer::imgui_tools;
    pub use crate::renderer::uniform::CameraUniform;
}
use crate::entities::bounding_box::BoundingBox;

pub mod colors {
    pub const SILVER:[f32;3] = [0.7, 0.7, 0.7];
    pub const CYAN_COLOR:[f32;3] = [0.0, 1.0, 1.0];
    pub const YELLOW_COLOR:[f32;3] = [1.0, 0.5, 1.0];
    pub const LIGHT_YELLOW_COLOR:[f32; 3] = [1.0, 0.9, 0.5];
    pub const RED_COLOR:[f32; 3] = [0.8, 0.3, 0.2]; 
    pub const GREEN_COLOR:[f32; 3] = [0.2, 0.8, 0.3]; 
    pub const BLUE_COLOR:[f32; 3] = [0.2, 0.3, 0.8]; 
    pub const CLEAR_COLOR:[f32; 3] = [0.1, 0.1, 0.1]; 
}

#[derive(Clone, Copy, Debug)]
pub struct Globals {
    pub ibl_enable: bool,
    pub skybox_enable: bool,
    pub exposure: f32,
    pub tonemap_filter: u32,
    pub axis_enable: bool,
    pub bbox_enable: bool,
}
impl Default for Globals {
    fn default() -> Self {
        Self {
            ibl_enable: true,
            skybox_enable: true,
            exposure: 1.0,
            tonemap_filter: 0,
            axis_enable: true,
            bbox_enable: true,
        }
    }
}

///shader: [pbr, blinnphong, light] 
#[repr(C, align(16))]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Light {
    color: [f32; 3],
    directional: u32,
    position: [f32; 3],
    cast_shadow: u32,
    entity_id: u64,
    pad2: [i32; 2],  
}
impl Default for Light {
    fn default() -> Self {
        Self {
            color: [1.0, 1.0, 1.0],
            cast_shadow: 0,
            directional: 1,
            position: [0.0, 0.0, -1.0],
            entity_id: 0,
            pad2: [0, 0],
        }
    }
}

// Ecs Components
#[derive(Default, Clone)]
pub struct LightComponent {
    pub data: Light,
}

pub struct MeshComponent {
    data: assets::mesh::Mesh,
}

#[derive(Clone)]
pub struct TransformComponent {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
}

pub struct TagComponent {
    pub name: String,
}

pub struct BoundingBoxComponent {
    pub bounding_box: BoundingBox,
    pub vertex_buffer: wgpu::Buffer,
}
