use super::*;

use crate::gpu::pipeline_manager::PipelineManager;
use crate::gpu::{BufferKind, GpuCache, GpuContext, GpuManager, ShadowManager, PickObject};
use crate::renderer::framebuilder::DrawStats;
use crate::renderer::uniform::{CameraUniform, GlobalUniform};

use legion::Entity;
use wgpu::Device;

use crate::camera::Camera;
use crate::globals::Globals;
use crate::prelude::{debug, info};
use crate::renderer::renderpass::*;

pub struct SceneRenderContext<'a> {
    pub gpu_context: &'a GpuContext,
    pub gpu_manager: &'a mut GpuManager,
    pub shadow_manager: &'a mut ShadowManager,
    pub pipeline_manager: &'a PipelineManager,
    pub gpu_cache: &'a mut GpuCache,
    pub pickobject: &'a mut PickObject,
}

pub struct RenderContext<'a> {
    pub device: &'a Device,
    pub gpu_cache: &'a GpuCache,

    pub gpu_mgr: &'a GpuManager,
    pub shadow_mgr: &'a ShadowManager,
    pub pip_mgr: &'a PipelineManager,
    pub pickobject: &'a mut PickObject,
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
            RenderPassEnum::PickObject(PickObjectPass::new()),
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
        runtime: &mut SceneRenderContext,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        size: (u32, u32),
        frame: &FrameData,
        camera: &Camera,
        globals: &Globals,
        selected: Option<Entity>,
    ) {
        let SceneRenderContext {
            gpu_context,
            gpu_manager,
            shadow_manager,
            pipeline_manager,
            gpu_cache,
            pickobject,
        } = runtime;

        let mut ctx = RenderContext {
            device: &gpu_context.device,
            gpu_cache: &gpu_cache,
            gpu_mgr: &gpu_manager,
            shadow_mgr: &shadow_manager,
            pip_mgr: &pipeline_manager,
            pickobject,
            target,
        };

        // Update Light to gpu
        if let Some(light_uniform) = frame.lights {
            gpu_manager.update_buffer(
                &gpu_context.queue,
                BufferKind::Lights,
                std::slice::from_ref(&light_uniform),
            );
        }

        // Update camera uniform buffer data to gpu
        let uniform = CameraUniform::from_camera_size(camera, size);
        gpu_manager.update_buffer(
            &gpu_context.queue,
            BufferKind::Camera,
            std::slice::from_ref(&uniform),
        );

        // Update global uniform buffer data to gpu
        use crate::EntityRawU64;
        let entity_id = selected.map(|id| id.as_raw_u64()).unwrap_or(0);
        let uniform = GlobalUniform::from_global_id(globals, entity_id);
        gpu_manager.update_buffer(
            &gpu_context.queue,
            BufferKind::Globals,
            std::slice::from_ref(&uniform),
        );

        // Update vertex instance buffer data to gpu
        gpu_manager.update_buffer(
            &gpu_context.queue,
            BufferKind::Instances,
            frame.instances.as_slice(),
        );

        for pass in &mut self.default_pass {
            pass.execute(encoder, &mut ctx, &frame);
        }

        self.stats = FrameStats {
            opaque: frame.opaque_stats,
            transmission: frame.transmission_stats,
        };
    }
}
