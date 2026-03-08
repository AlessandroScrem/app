
pub trait InternalCounter {
    fn internal_counter(&self) -> GpuInternalCounters;
}

#[derive(Default)]
pub struct GpuInternalCounters {
    pub textures: GpuResourceStats,
    pub materials: GpuResourceStats,
    pub meshes: GpuResourceStats,
}

pub trait HasGpuStats {
    fn get_stats(&self) -> GpuResourceStats;
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

// TODO!"not yet implemented on wgpu 25.0"
// 
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