use crate::app::application::AppRenderData;
use crate::ecs::entity_id::EntityRawU64;
use crate::engine::gpu_sync::TextureLoadService;
use crate::gpu::{BufferKind, GpuCache, GpuContext, GpuManager, GpuSurface};
use crate::renderer::FrameData;
use crate::renderer::framebuilder::{FrameBuilder, FrameTasks};
use crate::renderer::uniform::{CameraUniform, GlobalUniform};

pub(crate) fn prepare_frame_data(
    render_data: AppRenderData,
    gpu_context: &GpuContext,
    gpu_surface: &GpuSurface,
    gpu_cache: &GpuCache,
    texture_loader: &TextureLoadService,
    gpu_manager: &GpuManager,
) -> FrameData {
    let AppRenderData {
        render_objects,
        asset_mgr,
        camera,
        globals,
        selected,
    } = render_data;
    let frame = FrameBuilder::prepare(
        render_objects,
        asset_mgr,
        globals,
        gpu_cache,
        texture_loader,
    );
    let config = gpu_surface.get_config();
    let camera_uniform = CameraUniform::from_camera_size(camera, (config.width, config.height));
    let global_uniform =
        GlobalUniform::from_global_id(globals, selected.map(|id| id.as_raw_u64()).unwrap_or(0));

    gpu_manager.update_buffer(
        &gpu_context.queue,
        BufferKind::Lights,
        std::slice::from_ref(&frame.light_uniform),
    );
    gpu_manager.update_buffer(
        &gpu_context.queue,
        BufferKind::Camera,
        std::slice::from_ref(&camera_uniform),
    );
    gpu_manager.update_buffer(
        &gpu_context.queue,
        BufferKind::Globals,
        std::slice::from_ref(&global_uniform),
    );
    gpu_manager.update_buffer(
        &gpu_context.queue,
        BufferKind::Instances,
        frame.instances.as_slice(),
    );
    gpu_manager.update_buffer(
        &gpu_context.queue,
        BufferKind::Lines,
        frame.lines.as_slice(),
    );

    let tasks = FrameTasks {
        axis_enable: globals.axis_enable,
        build_mips_cp: globals.mips_cp,
        entity_selected: selected,
        skybox_enable: globals.skybox_enable,
        skybox_blur: globals.skybox_enable_blur,
    };

    FrameData {
        opaque_batches: frame.opaque_batches,
        transmission_batches: frame.transmission_batches,
        transmission_stats: frame.transmission_stats,
        lights: Some(frame.light_uniform),
        lines: frame.lines,
        opaque_stats: frame.opaque_stats,
        tasks,
    }
}
