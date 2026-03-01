pub(crate) mod gpu_manager;
pub(crate) mod gpu_material_cache;
pub(crate) mod gpu_mesh_cache;
pub(crate) mod gpu_texture_cache;
pub(crate) mod hdr_frame;
pub(crate) mod imgui_renderer;
pub(crate) mod light_manager;
pub(crate) mod pipeline_manager;
pub(crate) mod renderer;
pub(crate) mod renderpass;
pub(crate) mod skybox_manager;
pub(crate) mod texture;
pub(crate) mod uniform;

pub(crate) use gpu_manager::{GpuManager, LayoutKind};
pub(crate) use hdr_frame::{HdrFrame, ObjectIDTexture};
pub(crate) use light_manager::LightManager;
pub(crate) use pipeline_manager::PipelineManager;
pub(crate) use skybox_manager::SkyboxManager;

pub(crate) use crate::assets::*;
pub(crate) use gpu_material_cache::*;
pub(crate) use gpu_mesh_cache::*;
pub(crate) use gpu_texture_cache::*;
pub(crate) use imgui_renderer::{ImguiRender, UiTexture, UiTextureResolver};
pub(crate) use texture::GpuTexture;

pub use renderer::Renderer;
pub use uniform::{CameraUniform, GlobalUniform, LightUniform, MaterialUniform, ModelUniform};

pub trait InternalCounter {
    fn internal_counter(&self) -> GpuInternalCounters;
}

#[derive(Default)]
pub struct GpuInternalCounters {
    pub textures: GpuResourceStats,
    pub materials: GpuResourceStats,
    pub meshes: GpuResourceStats,
}

#[derive(Default, Debug, Clone)]
pub struct GpuResourceStats {
    pub count: usize,
    pub estimated_bytes: usize,
}

impl GpuResourceStats {
    pub fn add(&mut self, size: usize) {
        self.estimated_bytes += size;
        self.count += 1;
    }
    pub fn remove(&mut self, size: usize) {
        if self.count > 0 {
            let result = self.estimated_bytes.checked_sub(size).unwrap_or(0);
            self.estimated_bytes = result;
            self.count -= 1;
        }
    }
}

pub trait HasGpuStats {
    fn get_stats(&self) -> GpuResourceStats;
}



// #[allow(dead_code)]
// #[derive(Default, Debug)]
// pub struct GpuInternalCounters {
//     // API objects
//     pub buffers: isize,
//     pub textures: isize,
//     pub texture_views: isize,
//     pub bind_groups: isize,
//     pub bind_group_layouts: isize,
//     pub render_pipelines: isize,
//     pub compute_pipelines: isize,
//     pub pipeline_layouts: isize,
//     pub samplers: isize,
//     pub command_encoders: isize,
//     pub shader_modules: isize,
//     pub query_sets: isize,
//     pub fences: isize,

//     // Resources
//     /// Amount of allocated gpu memory attributed to buffers, in bytes.
//     pub buffer_memory: isize,
//     /// Amount of allocated gpu memory attributed to textures, in bytes.
//     pub texture_memory: isize,
//     /// Amount of allocated gpu memory attributed to acceleration structures, in bytes.
//     pub acceleration_structure_memory: isize,
//     /// Number of gpu memory allocations.
//     pub memory_allocations: isize,
// }

// impl From<wgpu::InternalCounters> for GpuInternalCounters {
//     fn from(ic: wgpu::InternalCounters) -> Self {
//         let hal = ic.hal;
//         Self {
//             buffers: hal.buffers.read(),
//             textures: hal.textures.read(),
//             texture_views: hal.texture_views.read(),
//             bind_groups: hal.bind_groups.read(),
//             bind_group_layouts: hal.bind_group_layouts.read(),
//             render_pipelines: hal.render_pipelines.read(),
//             compute_pipelines: hal.compute_pipelines.read(),
//             pipeline_layouts: hal.pipeline_layouts.read(),
//             samplers: hal.samplers.read(),
//             command_encoders: hal.command_encoders.read(),
//             shader_modules: hal.shader_modules.read(),
//             query_sets: hal.query_sets.read(),
//             fences: hal.fences.read(),

//             buffer_memory: hal.buffer_memory.read(),
//             texture_memory: hal.texture_memory.read(),
//             acceleration_structure_memory: hal.acceleration_structure_memory.read(),
//             memory_allocations: hal.memory_allocations.read(),
//         }
//     }
// }
