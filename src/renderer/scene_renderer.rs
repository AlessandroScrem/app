use super::*;

use crate::gpu::pipeline_manager::PipelineManager;
use crate::gpu::{GpuCache, GpuContext, GpuManager, ShadowManager};
use crate::renderer::framebuilder::DrawStats;

use wgpu::Device;

use crate::prelude::{debug, info};
use crate::renderer::renderpass::*;

pub struct SceneRenderContext<'a> {
    pub gpu_context: &'a GpuContext,
    pub gpu_manager: &'a  GpuManager,
    pub shadow_manager: &'a ShadowManager,
    pub pipeline_manager: &'a PipelineManager,
    pub gpu_cache: &'a GpuCache,
}

pub struct RenderContext<'a> {
    pub device: &'a Device,
    pub gpu_cache: &'a GpuCache,

    pub gpu_mgr: &'a GpuManager,
    pub shadow_mgr: &'a ShadowManager,
    pub pip_mgr: &'a PipelineManager,
    pub target: &'a wgpu::TextureView,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct FrameStats {
    pub opaque: DrawStats,
    pub transmission: DrawStats,
}

pub struct SceneRenderer {
    default_pass: Vec<RenderPassEnum>,
    stats: FrameStats,
}

impl SceneRenderer {
    pub fn new() -> Self {
        let timer = std::time::Instant::now();
        info!("Initializing renderer...");

        debug!("Renderer initialized in {} ms", timer.elapsed().as_millis());

        let default_pass = vec![
            RenderPassEnum::Shadow(ShadowPass {}),
            RenderPassEnum::Mesh(MeshPass::opaque()),
            RenderPassEnum::Skybox(SkyboxPass::new()),
            RenderPassEnum::BuildMipmaps(BuildMipmapsPass::new()),
            RenderPassEnum::Transmission(MeshPass::transmission()),
            RenderPassEnum::LightsIcon(LightsIconPass::new()),
            RenderPassEnum::Axis(AxisPass::new()),
            RenderPassEnum::Lines(LinesPass::new()),
            RenderPassEnum::Linearize(LinearizePass::new()),
            RenderPassEnum::Outline(OutlinePass::new()),
        ];

        Self {
            default_pass,
            stats: FrameStats::default(),
        }
    }

    pub fn get_render_stats(&self) -> FrameStats {
        self.stats
    }

    pub fn render(
        &mut self,
        runtime: &SceneRenderContext,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        frame: &FrameData,
    ) {
        let SceneRenderContext {
            gpu_context,
            gpu_manager,
            shadow_manager,
            pipeline_manager,
            gpu_cache,
        } = runtime;

        let mut ctx = RenderContext {
            device: &gpu_context.device,
            gpu_cache: &gpu_cache,
            gpu_mgr: &gpu_manager,
            shadow_mgr: &shadow_manager,
            pip_mgr: &pipeline_manager,
            target,
        };

        for pass in &mut self.default_pass {
            pass.execute(encoder, &mut ctx, &frame);
        }

        self.stats = FrameStats {
            opaque: frame.opaque_stats,
            transmission: frame.transmission_stats,
        };
    }
}
