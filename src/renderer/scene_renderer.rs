use std::collections::HashSet;

use super::*;

use crate::app::app_impl::RuntimeContext;
use crate::assets::asset_manager::AssetManager;
use crate::entities::EntityRawU64;
use crate::gpu::GpuContext;
use crate::renderer::framebuilder::DrawStats;
use crate::uniform::{CameraUniform, GlobalUniform};

use legion::{Entity, World};
use wgpu::{Device, Queue};

use crate::picking::PickObject;
use crate::renderer::renderpass::*;

use crate::Globals;

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

const MAX_INSTANCES: usize = 1000;

#[derive(Debug, Default, Copy, Clone)]
pub struct FrameStats {
    pub opaque: DrawStats,
    pub transmission: DrawStats,
}

pub struct SceneRenderer {
    pickobject: PickObject,
    default_pass: Vec<RenderPassEnum>,
    instance_buffer: wgpu::Buffer,
    stats: FrameStats,
}

impl SceneRenderer {
    pub fn new(gpu_context: &GpuContext) -> Self {
        let timer = std::time::Instant::now();
        info!("Initializing renderer...");

        let device = &gpu_context.device;
        let pickobject = PickObject::new(&device);

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
            pickobject,
            default_pass,
            instance_buffer,
            stats: FrameStats::default(),
        }
    }

    pub fn get_hovered(&mut self, gpu_context: &GpuContext) -> Option<Entity> {
        self.pickobject.poll_readback(&gpu_context.device)
    }

    pub fn get_render_stats(&self) -> FrameStats {
        self.stats
    }

    pub fn render(
        &mut self,
        runtime: &mut RuntimeContext,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        size: (u32, u32),
        asset_mgr: &AssetManager,
        world: &World,
        camera: &Camera,
        globals: &Globals,
        selected: Option<Entity>,
    ) {
        let RuntimeContext {
            gpu_context,
            gpu_manager,
            pipeline_manager,
            gpu_cache,
            input,
        } = runtime;

        // sync GpuCache Ids with assets Ids (meshes materials textures)
        // update skybox
        gpu_manager.sync_ibl(gpu_cache, gpu_context, asset_mgr);
        gpu_cache.sync_caches(gpu_context, gpu_manager, asset_mgr);

        let frame = FrameBuilder::build(
            world,
            &gpu_context.device,
            asset_mgr,
            selected,
            &self.pickobject,
            input,
            globals,
        );
        let mut ctx = RenderContext {
            device: &gpu_context.device,
            queue: &gpu_context.queue,
            gpu_cache: &gpu_cache,

            gpu_mgr: &gpu_manager,
            pip_mgr: &pipeline_manager,
            pickobject: &self.pickobject,
            target: &target,
            instance_buffer: &self.instance_buffer,
        };

        // Update uniform buffer data to gpu
        self.update_render_globals_to_gpu(&mut ctx, camera, globals, selected, size);
        Self::update_meshes_materials_to_gpu(&mut ctx, asset_mgr, &frame);
        Self::update_lights_to_gpu(gpu_context, gpu_manager, &frame);

        // Update vertex instance buffer data to gpu
        Self::update_vertex_instances_to_gpu(&mut ctx, &frame);

        for pass in &mut self.default_pass {
            pass.execute(encoder, &mut ctx, &frame);
        }

        self.stats = FrameStats {
            opaque: frame.opaque_stats,
            transmission: frame.transmission_stats,
        };
    }

    // TODO! move out of here
    fn update_render_globals_to_gpu(
        &self,
        ctx: &mut RenderContext,
        camera: &Camera,
        globals: &Globals,
        selected: Option<Entity>,
        size: (u32, u32),
    ) {
        let entity_id = match selected {
            Some(id) => (&id).as_raw_u64(),
            None => 0,
        };

        let queue = ctx.queue;

        ctx.gpu_mgr
            .update_camera(queue, &CameraUniform::from_camera_size(camera, size));
        ctx.gpu_mgr
            .update_globals(queue, &GlobalUniform::from_global_id(globals, entity_id));
    }

    // TODO! move out of here
    fn update_vertex_instances_to_gpu(ctx: &mut RenderContext, frame: &FrameData) {
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

    // TODO! move out of here
    fn update_meshes_materials_to_gpu(
        ctx: &mut RenderContext,
        asset_mgr: &AssetManager,
        frame: &FrameData,
    ) {
        // -------- Mesh --------
        let queue = &ctx.queue;
        let gpu_cache = &ctx.gpu_cache;

        let mut updated_materials = HashSet::new();

        fn gpu_update(
            asset_mgr: &AssetManager,
            gpu_cache: &GpuCache,
            queue: &Queue,
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

    // TODO! move out of here
    fn update_lights_to_gpu(gpu_context: &GpuContext, gpu_manager: &GpuManager, frame: &FrameData) {
        let queue = &gpu_context.queue;

        if let Some(light_uniform) = frame.lights {
            queue.write_buffer(
                gpu_manager.get_buffer(BufferKind::Light),
                0,
                bytemuck::bytes_of(&light_uniform),
            );
        }
    }
}
