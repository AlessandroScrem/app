use crate::gpu::pipeline_manager::PipelineManager;
use crate::gpu::{GpuCache, GpuContext, GpuManager, ShadowManager};
use crate::prelude::{debug, info};
use crate::renderer::framebuilder::{DrawStats, FrameData};
use crate::renderer::rendergraph::RenderGraph;
use crate::renderer::renderpass::*;
use wgpu::Device;

pub struct SceneRenderContext<'a> {
    pub gpu_context: &'a GpuContext,
    pub gpu_manager: &'a GpuManager,
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
    render_graph: RenderGraph,
    stats: FrameStats,
}

impl SceneRenderer {
    pub fn new() -> Self {
        let timer = std::time::Instant::now();
        info!("Initializing renderer...");

        debug!("Renderer initialized in {} ms", timer.elapsed().as_millis());

        let mut render_graph = RenderGraph::new();
        render_graph.add_pass(RenderPassEnum::Shadow(ShadowPass {}));
        render_graph.add_pass(RenderPassEnum::Mesh(MeshPass::opaque()));
        render_graph.add_pass(RenderPassEnum::Skybox(SkyboxPass {}));
        render_graph.add_pass(RenderPassEnum::BuildMipmaps(BuildMipmapsPass {}));
        render_graph.add_pass(RenderPassEnum::Transmission(MeshPass::transmission()));
        render_graph.add_pass(RenderPassEnum::LightsIcon(LightsIconPass {}));
        render_graph.add_pass(RenderPassEnum::Axis(AxisPass {}));
        render_graph.add_pass(RenderPassEnum::Lines(LinesPass {}));
        render_graph.add_pass(RenderPassEnum::Linearize(LinearizePass {}));
        render_graph.add_pass(RenderPassEnum::Outline(OutlinePass {}));

        Self {
            render_graph,
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

        if let Err(error) = self.render_graph.execute(encoder, &mut ctx, frame) {
            log::error!("Render graph compilation failed; frame rendering skipped: {error}");
            return;
        }

        self.stats = FrameStats {
            opaque: frame.opaque_stats,
            transmission: frame.transmission_stats,
        };
    }
}
