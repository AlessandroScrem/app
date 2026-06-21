use crate::entities::EntityRawU64;
use crate::gpu::GpuManager;
use crate::prelude::*;
use crate::globals::Globals;
use crate::renderer::scene_renderer::{MAX_INSTANCES, RenderContext};
use crate::renderer::uniform::{CameraUniform, GlobalUniform};
use legion::Entity;

use super::FrameData;

pub struct GpuSync;

impl GpuSync {

    pub fn update_camera_and_globals_to_gpu(
        ctx: &mut RenderContext,
        camera: &Camera,
        globals: &Globals,
        selected: Option<Entity>,
        size: (u32, u32),
    ) {
        let entity_id = selected.map(|id| id.as_raw_u64()).unwrap_or(0);
        let queue = ctx.queue;

        ctx.gpu_mgr
            .update_camera(queue, &CameraUniform::from_camera_size(camera, size));
        ctx.gpu_mgr
            .update_globals(queue, &GlobalUniform::from_global_id(globals, entity_id));
    }

    pub fn update_vertex_instances_to_gpu(ctx: &mut RenderContext, frame: &FrameData) {
        assert!(
            frame.instances.len() <= MAX_INSTANCES,
            "Too many instances! Max is {}",
            MAX_INSTANCES
        );

        ctx.queue.write_buffer(
            ctx.instance_buffer,
            0,
            bytemuck::cast_slice(&frame.instances),
        );
    }

    pub fn update_lights_to_gpu(queue: &wgpu::Queue, gpu_manager: &GpuManager, frame: &FrameData) {
        if let Some(light_uniform) = frame.lights {
            queue.write_buffer(
                gpu_manager.get_buffer(crate::gpu::BufferKind::Light),
                0,
                bytemuck::bytes_of(&light_uniform),
            );
        }
    }
}
