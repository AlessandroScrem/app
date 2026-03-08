use super::*;

use crate::assets::asset_manager::AssetManager;
use crate::entities::EntityRawU64;
use crate::gpu::{GpuContext, GpuSurface};
use crate::input::Input;
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
    gpu_mgr: GpuManager,
    pipeline_mgr: PipelineManager,
    skybox_mgr: SkyboxManager,

    pickobject: PickObject,
    passes: Vec<RenderPassEnum>,
}

impl SceneRenderer {
    pub fn new(
        gpu_context: &GpuContext,
        gpu_surface: &GpuSurface,
        gpu_cache: &mut GpuCache,
        asset_mgr: &mut AssetManager,
    ) -> Self {
        let timer = std::time::Instant::now();
        info!("Initializing renderer...");

        let device = &gpu_context.device;
        let queue = &gpu_context.queue;
        let width = gpu_surface.get_config().width;
        let height = gpu_surface.get_config().height;
        let format = gpu_surface.get_config().format;

        let gpu_mgr = GpuManager::new(&device, &queue, width, height);
        let pipeline_mgr = PipelineManager::new(&device, &gpu_mgr, format);
        let pickobject = PickObject::new(&device);

        // Skybox initialization
        let hdr_id = asset_mgr.skybox.get_id();
        let hdr = gpu_cache.textures.get_or_fallback(hdr_id);
        let skybox_mgr = SkyboxManager::new(hdr_id, hdr, &device, &queue, &gpu_mgr);
        // -----

        debug!("Renderer initialized in {} ms", timer.elapsed().as_millis());

        let passes = vec![
            RenderPassEnum::Mesh(MeshPass::new()),
            RenderPassEnum::Light(LightPass::new()),
            RenderPassEnum::Skybox(SkyboxPass::new()),
            RenderPassEnum::Axis(AxisPass::new()),
            RenderPassEnum::BBox(BBoxPass::new()),
            RenderPassEnum::Linearize(LinearizePass::new()),
            RenderPassEnum::Outline(OutlinePass::new()),
            RenderPassEnum::PickObject(PickObjectPass::new()),
        ];

        Self {
            gpu_mgr,
            pipeline_mgr,
            skybox_mgr,
            pickobject,
            passes,
        }
    }

    pub fn get_hovered(&mut self, gpu_context: &GpuContext) -> Option<Entity> {
        self.pickobject.poll_readback(&gpu_context.device)
    }


    fn sync_skybox(
        &mut self,
        gpu_cache: &mut GpuCache,
        gpu_context: &GpuContext,
        asset_mgr: &AssetManager,
    ) {
        if asset_mgr.skybox.get_id() != self.skybox_mgr.get_hdr_id() {
            let hdr_texture = gpu_cache
                .textures
                .get_or_fallback(asset_mgr.skybox.get_id());
            self.skybox_mgr.update_skybox(
                asset_mgr.skybox.get_id(),
                hdr_texture,
                &gpu_context.device,
                &gpu_context.queue,
                &self.gpu_mgr,
            );
        }
    }

    /// update skybox
    /// sync GpuCache Ids with assets Ids (meshes materials textures)
    pub fn prepare(
        &mut self,
        gpu_cache: &mut GpuCache,
        gpu_context: &GpuContext,
        asset_mgr: &AssetManager,
    ) {
        self.sync_skybox(gpu_cache, gpu_context, asset_mgr);
        gpu_cache.sync_caches(gpu_context, &self.gpu_mgr, asset_mgr);
    }

    pub fn render(
        &mut self,
        gpu_context: &GpuContext,
        gpu_cache: &mut GpuCache,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        size: (u32, u32),
        asset_mgr: &AssetManager,
        world: &World,
        camera: &Camera,
        globals: &Globals,
        selected: Option<Entity>,
        input: &Input,
    ) {
        // sync GpuCache Ids with assets Ids (meshes materials textures)
        self.prepare(gpu_cache, gpu_context, asset_mgr);

        // update global data (uniform) to GPU
        self.update_render_globals_to_gpu(gpu_context, camera, globals, selected, size);

        let mut ctx = RenderContext {
            device: &gpu_context.device,
            queue: &gpu_context.queue,
            gpu_cache: &gpu_cache,

            gpu_mgr: &self.gpu_mgr,
            pip_mgr: &self.pipeline_mgr,
            skb_mgr: &self.skybox_mgr,
            pickobject: &self.pickobject,
            target: &target,
        };

        // Update world buffer data to gpu
        for pass in &mut self.passes {
            pass.prepare(
                asset_mgr, world, camera, globals, selected, input, &mut ctx,
            );
        }

        for pass in &mut self.passes {
            pass.execute(encoder, &mut ctx, &asset_mgr);
        }
    }

    fn update_render_globals_to_gpu(
        &self,
        gpu_context: &GpuContext,
        camera: &Camera,
        globals: &Globals,
        selected: Option<Entity>,
        size: (u32, u32),
    ) {
        let entity_id = match selected {
            Some(id) => (&id).as_raw_u64(),
            None => 0,
        };

        let screen_size = [size.0 as f32, size.1 as f32];
        let updated_camera_uniform = CameraUniform {
            view_position: camera.get_position().to_homogeneous().into(),
            view: camera.get_view_mat().into(),
            proj: camera.get_projection_mat().into(),
            screen_size,
            ..Default::default()
        };

        let updated_globals_uniform = GlobalUniform {
            ibl_enable: globals.ibl_enable as u32,
            skybox_enable: globals.skybox_enable as u32,
            exposure: globals.exposure,
            ibl_intensity: globals.ibl_intensity,
            tonemap_filter: globals.tonemap_filter,
            entity_id,
            debug: globals.debug_code,
            ..Default::default()
        };

        gpu_context.queue.write_buffer(
            self.gpu_mgr.get_buffer(BufferKind::Camera),
            0,
            bytemuck::bytes_of(&updated_camera_uniform),
        );
        gpu_context.queue.write_buffer(
            self.gpu_mgr.get_buffer(BufferKind::Globals),
            0,
            bytemuck::bytes_of(&updated_globals_uniform),
        );
    }

    pub fn resize_frame(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        // resize gpu_manager
        self.gpu_mgr.resize_frame(device, width, height);
    }
}
