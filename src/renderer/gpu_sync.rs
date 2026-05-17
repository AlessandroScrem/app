use crate::assets::{MaterialId, asset_manager::AssetManager};
use crate::entities::EntityRawU64;
use crate::gpu::{GpuCache, GpuContext, GpuManager};
use crate::prelude::*;
use crate::renderer::scene_renderer::{MAX_INSTANCES, RenderContext};
use crate::uniform::{CameraUniform, GlobalUniform};
use legion::Entity;

use super::FrameData;

pub struct GpuSync;

impl GpuSync {
    pub fn sync_caches(
        gpu_cache: &mut GpuCache,
        gpu_context: &GpuContext,
        gpu_manager: &mut GpuManager,
        asset_mgr: &AssetManager,
    ) {
        gpu_manager.sync_ibl(gpu_cache, gpu_context, asset_mgr);
        gpu_cache.sync_caches(gpu_context, gpu_manager, asset_mgr);
    }

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

    pub fn update_meshes_materials_to_gpu(
        queue: &wgpu::Queue,
        gpu_cache: &GpuCache,
        asset_mgr: &AssetManager,
        frame: &FrameData,
    ) {
        let mut updated_materials = std::collections::HashSet::new();

        fn gpu_update(
            asset_mgr: &AssetManager,
            gpu_cache: &GpuCache,
            queue: &wgpu::Queue,
            material_id: MaterialId,
        ) {
            if let Some(material_desc) = asset_mgr.materials.get_desc(material_id) {
                let updated_uniform = uniform::MaterialUniform::from(material_desc);
                gpu_cache
                    .material
                    .update(&material_id, queue, &updated_uniform);
            }
        }

        for batch in frame.opaque_batches.iter() {
            if updated_materials.insert(batch.material) {
                gpu_update(asset_mgr, gpu_cache, queue, batch.material);
            }
        }

        for batch in frame.transmission_batches.iter() {
            if updated_materials.insert(batch.material) {
                gpu_update(asset_mgr, gpu_cache, queue, batch.material);
            }
        }
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
