use super::gpu_sync::GpuSync;
use super::*;

use crate::gpu::GpuContext;
use crate::renderer::framebuilder::DrawStats;

use legion::Entity;
use wgpu::{Device, Queue};

use crate::picking::PickObject;
use crate::renderer::renderpass::*;

use crate::Globals;

pub struct SceneRenderContext<'a> {
    pub gpu_context: &'a GpuContext,
    pub gpu_manager: &'a mut GpuManager,
    pub pipeline_manager: &'a PipelineManager,
    pub gpu_cache: &'a mut GpuCache,
    pub pickobject: &'a PickObject,
}

pub struct RenderContext<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,
    pub gpu_cache: &'a GpuCache,

    pub gpu_mgr: &'a GpuManager,
    pub pip_mgr: &'a PipelineManager,
    pub pickobject: &'a PickObject,
    pub target: &'a wgpu::TextureView,
    pub instance_buffer: &'a wgpu::Buffer,
}

pub(crate) const MAX_INSTANCES: usize = 1000;

#[derive(Debug, Default, Copy, Clone)]
pub struct FrameStats {
    pub opaque: DrawStats,
    pub transmission: DrawStats,
}

pub struct SceneRenderer {
    default_pass: Vec<RenderPassEnum>,
    instance_buffer: wgpu::Buffer,
    stats: FrameStats,
}

impl SceneRenderer {
    pub fn new(gpu_context: &GpuContext) -> Self {
        let timer = std::time::Instant::now();
        info!("Initializing renderer...");

        let device = &gpu_context.device;

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (std::mem::size_of::<vertexdata::VertexInstance>() * MAX_INSTANCES) as u64, // TODO! dynamic
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        debug!("Renderer initialized in {} ms", timer.elapsed().as_millis());

        let default_pass = vec![
            RenderPassEnum::Mesh(MeshPass::opaque()),
            RenderPassEnum::Skybox(SkyboxPass::new()),
            RenderPassEnum::BuildMipmaps(BuildMipmapsPass::new()),
            RenderPassEnum::Transmission(MeshPass::transmission()),
            RenderPassEnum::Light(LightPass::new()),
            RenderPassEnum::Axis(AxisPass::new()),
            RenderPassEnum::BBox(BoundingboxPass::new()),
            RenderPassEnum::Linearize(LinearizePass::new()),
            RenderPassEnum::Outline(OutlinePass::new()),
            RenderPassEnum::PickObject(PickObjectPass::new()),
        ];

        Self {
            default_pass,
            instance_buffer,
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
            pipeline_manager,
            gpu_cache,
            pickobject,
        } = runtime;

        let mut ctx = RenderContext {
            device: &gpu_context.device,
            queue: &gpu_context.queue,
            gpu_cache: &gpu_cache,

            gpu_mgr: &gpu_manager,
            pip_mgr: &pipeline_manager,
            pickobject,
            target,
            instance_buffer: &self.instance_buffer,
        };

        // Update uniform buffer data to gpu
        GpuSync::update_camera_and_globals_to_gpu(&mut ctx, camera, globals, selected, size);

        // Update vertex instance buffer data to gpu
        GpuSync::update_vertex_instances_to_gpu(&mut ctx, &frame);

        for pass in &mut self.default_pass {
            pass.execute(encoder, &mut ctx, &frame);
        }

        self.stats = FrameStats {
            opaque: frame.opaque_stats,
            transmission: frame.transmission_stats,
        };
    }
}
