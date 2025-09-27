mod app;
mod application_handler;
pub mod assets;
mod camera;
mod entities;
pub mod input;
mod renderer;
mod scene;
pub mod systems;
pub mod transform;

pub mod test_utils;

pub mod prelude {
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
pub struct DeltaTime(pub f32);

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

#[repr(C)]
#[derive(Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlobalUniform {
    ibl_enable: u32,
    skybox_enable: u32,
    exposure: f32,
    tonemap_filter: u32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Light {
    color: [f32; 3],
    directional: u32,
    position: [f32; 3],
    cast_shadow: u32,
}
impl Default for Light {
    fn default() -> Self {
        Self {
            color: [1.0, 1.0, 1.0],
            cast_shadow: 0,
            directional: 1,
            position: [0.0, 0.0, -1.0],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct MaterialUniform {
    pub color: [f32; 4],
    pub roughness: f32,
    pub metallic: f32,
    pub roughness_use_texture: u32,
    pub metallic_use_texture: u32,
    pub color_use_texture: u32,
    pub padding: [u32; 3],
}
impl Default for MaterialUniform {
    fn default() -> Self {
        Self {
            color: [1.0, 1.0, 1.0, 1.0],
            roughness: 1.0,
            metallic: 0.0,
            roughness_use_texture: 0,
            metallic_use_texture: 0,
            color_use_texture: 0,
            padding: [0; 3],
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
