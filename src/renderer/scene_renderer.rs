use super::*;

use crate::app::app_impl::RuntimeContext;
use crate::assets::asset_manager::AssetManager;
use crate::entities::EntityRawU64;
use crate::gpu::GpuContext;
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
    pub skb_mgr: &'a SkyboxManager,
    pub pickobject: &'a PickObject,
    pub target: &'a wgpu::TextureView,
}

pub struct SceneRenderer {
    skybox_mgr: SkyboxManager,

    pickobject: PickObject,
    #[allow(unused)]
    default_pass: Vec<RenderPassEnum>,
    transmission_pass: Vec<RenderPassEnum>,
}

impl SceneRenderer {
    pub fn new(
        gpu_context: &GpuContext,
        gpu_manager: &mut GpuManager,
        gpu_cache: &mut GpuCache,
        asset_mgr: &mut AssetManager,
    ) -> Self {
        let timer = std::time::Instant::now();
        info!("Initializing renderer...");

        let device = &gpu_context.device;
        let queue = &gpu_context.queue;
        let pickobject = PickObject::new(&device);

        // Skybox initialization
        let hdr_id = asset_mgr.skybox.get_id();
        let hdr = gpu_cache.textures.get_or_fallback_white(hdr_id);
        let skybox_mgr = SkyboxManager::new(hdr_id, hdr, &device, &queue, gpu_manager);
        // -----

        debug!("Renderer initialized in {} ms", timer.elapsed().as_millis());

        let default_pass = vec![
            RenderPassEnum::Mesh(MeshPass::new()),
            RenderPassEnum::Light(LightPass::new()),
            RenderPassEnum::Skybox(SkyboxPass::new()),
            RenderPassEnum::Axis(AxisPass::new()),
            RenderPassEnum::BBox(BoundingboxPass::new()),
            RenderPassEnum::Linearize(LinearizePass::new()),
            RenderPassEnum::Outline(OutlinePass::new()),
            RenderPassEnum::PickObject(PickObjectPass::new()),
        ];

        let transmission_pass = vec![
            RenderPassEnum::Mesh(MeshPass::new()),
            RenderPassEnum::HdrMipmaps(HdrMipmapsPass::new()),
            RenderPassEnum::Skybox(SkyboxPass::new()),
            RenderPassEnum::Transmission(TransmissionPass::new()),
            RenderPassEnum::Light(LightPass::new()),
            RenderPassEnum::Axis(AxisPass::new()),
            RenderPassEnum::BBox(BoundingboxPass::new()),
            RenderPassEnum::Linearize(LinearizePass::new()),
            RenderPassEnum::Outline(OutlinePass::new()),
            RenderPassEnum::PickObject(PickObjectPass::new()),
        ];

        Self {
            skybox_mgr,
            pickobject,
            default_pass: default_pass,
            transmission_pass,
        }
    }

    pub fn get_hovered(&mut self, gpu_context: &GpuContext) -> Option<Entity> {
        self.pickobject.poll_readback(&gpu_context.device)
    }

    fn sync_skybox(
        &mut self,
        gpu_manager: &mut GpuManager,
        gpu_cache: &mut GpuCache,
        gpu_context: &GpuContext,
        asset_mgr: &AssetManager,
    ) {
        if asset_mgr.skybox.get_id() != self.skybox_mgr.get_hdr_id() {
            let hdr_texture = gpu_cache
                .textures
                .get_or_fallback_white(asset_mgr.skybox.get_id());
            self.skybox_mgr.update_skybox(
                asset_mgr.skybox.get_id(),
                hdr_texture,
                &gpu_context.device,
                &gpu_context.queue,
                gpu_manager,
            );
        }
    }

    // CHANGEME!
    pub fn update_ibl_bind_group(
        &mut self,
        gpu_manager: &mut GpuManager,
        gpu_context: &GpuContext,
    ) {
        self.skybox_mgr
            .update_ibl_bind_group(&gpu_context.device, gpu_manager);
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
        self.sync_skybox(gpu_manager, gpu_cache, gpu_context, asset_mgr);
        gpu_cache.sync_caches(gpu_context, gpu_manager, asset_mgr);

        // update global data (uniform) to GPU
        // TODO! move out of here
        self.update_render_globals_to_gpu(
            gpu_context,
            gpu_manager,
            camera,
            globals,
            selected,
            size,
        );

        let mut ctx = RenderContext {
            device: &gpu_context.device,
            queue: &gpu_context.queue,
            gpu_cache: &gpu_cache,

            gpu_mgr: &gpu_manager,
            pip_mgr: &pipeline_manager,
            skb_mgr: &self.skybox_mgr,
            pickobject: &self.pickobject,
            target: &target,
        };

        // Update world buffer data to gpu
        for pass in &mut self.transmission_pass {
            pass.prepare(asset_mgr, world, globals, selected, input, &mut ctx);
        }

        for pass in &mut self.transmission_pass {
            pass.execute(encoder, &mut ctx, &asset_mgr);
        }
    }

    // TODO! move out of here
    fn update_render_globals_to_gpu(
        &self,
        gpu_context: &GpuContext,
        gpu_manager: &GpuManager,
        camera: &Camera,
        globals: &Globals,
        selected: Option<Entity>,
        size: (u32, u32),
    ) {
        let entity_id = match selected {
            Some(id) => (&id).as_raw_u64(),
            None => 0,
        };

        gpu_manager.update_camera(
            &gpu_context.queue,
            &CameraUniform::from_camera_size(camera, size),
        );
        gpu_manager.update_globals(
            &gpu_context.queue,
            &GlobalUniform::from_global_id(globals, entity_id),
        );
    }
}
