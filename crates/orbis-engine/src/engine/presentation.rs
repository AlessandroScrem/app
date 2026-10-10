use crate::gpu::{GpuContext, GpuManager, GpuSurface};

/// Applies a non-zero window size to the GPU surface and its frame resources.
pub(crate) fn resize(
    gpu_context: &GpuContext,
    gpu_surface: &mut GpuSurface,
    gpu_manager: &mut GpuManager,
    width: u32,
    height: u32,
) {
    if width == 0 || height == 0 {
        return;
    }

    gpu_manager.resize_frame(&gpu_context.as_ref(), width, height);
    gpu_surface.resize_frame(&gpu_context.device, width, height);
}
