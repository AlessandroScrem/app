use super::*;

use crate::assets::asset_manager::AssetManager;
use crate::entities::EntityRawU64;
use crate::gpu::{GpuContext, GpuSurface, ImguiRender};
use crate::input::Input;

use legion::{Entity, Resources, World};
use wgpu::{Device, Queue, SurfaceConfiguration};

use crate::picking::PickObject;
use crate::renderer::renderpass::*;

use crate::Globals;

impl InternalCounter for Renderer {
    fn internal_counter(&self) -> GpuInternalCounters {
        GpuInternalCounters {
            textures: self.gpu_cache.textures.get_stats(),
            meshes: self.gpu_cache.mesh.get_stats(),
            materials: self.gpu_cache.material.get_stats(),
        }
    }
}

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

pub struct GpuCache {
    pub mesh: GpuMeshCache,
    pub material: GpuMaterialCache,
    pub textures: GpuTextureCache,
}

pub struct Renderer {
    gpu_mgr: GpuManager,
    pipeline_mgr: PipelineManager,
    skybox_mgr: SkyboxManager,

    pickobject: PickObject,
    passes: Vec<RenderPassEnum>,

    gpu_cache: GpuCache,
}

impl Renderer {
    pub fn new(
        gpu_context: &GpuContext,
        gpu_surface: &GpuSurface,
        // window: Arc<Window>,
        // imgui_ctx: &mut imgui::Context,
        asset_mgr: &mut AssetManager,
    ) -> Self {
        let timer = std::time::Instant::now();
        info!("Initializing renderer...");

        let device = &gpu_context.device;
        let queue = &gpu_context.queue;
        let width = gpu_surface.get_config().width;
        let height = gpu_surface.get_config().height;
        let format = gpu_surface.get_config().format;

        let mut texture_cache = GpuTextureCache::new(&device, &queue);
        texture_cache.upload_textures(&mut asset_mgr.textures, &device, &queue);
        
        let gpu_mgr = GpuManager::new(&device, &queue, width, height);
        let pipeline_mgr = PipelineManager::new(&device, &gpu_mgr, format);
        let pickobject = PickObject::new(&device);
        
        // Skybox initialization
        let hdr_id = asset_mgr.skybox.get_id();
        let hdr = texture_cache.get_or_fallback(hdr_id /* &device, &queue */);
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
            // RenderPassEnum::Imgui(ImguiPass::new()),
        ];
        
        let gpu_cache = GpuCache {
            textures: texture_cache,
            material: GpuMaterialCache::default(),
            mesh: GpuMeshCache::default(),
        };

        Self {
            gpu_mgr,
            pipeline_mgr,
            skybox_mgr,
            pickobject,
            passes,
            gpu_cache,
        }
    }

    pub fn get_hovered(&mut self, gpu_context: &GpuContext) -> Option<Entity> {
        self.pickobject.poll_readback(&gpu_context.device)
    }

    pub fn sync_imgui_texture(&mut self, gpu_context: &GpuContext, imgui_render: &mut ImguiRender) {
        let registry = &mut imgui_render.registry;
        let renderer = &mut imgui_render.renderer;
        let texture_cache = &self.gpu_cache.textures;
        let device = &gpu_context.device;

        // record new textures
        use imgui_wgpu::RawTextureConfig;
        for (gpu_id, tex) in texture_cache.iter() {
            if !registry.ids.contains_key(&gpu_id) {
                let texture_config = RawTextureConfig {
                    label: None,
                    sampler_desc: wgpu::SamplerDescriptor {
                        mag_filter: wgpu::FilterMode::Linear,
                        min_filter: wgpu::FilterMode::Linear,
                        mipmap_filter: wgpu::FilterMode::Linear,
                        ..Default::default()
                    },
                };
                let id = renderer
                    .textures
                    .insert(imgui_wgpu::Texture::from_raw_parts(
                        device,
                        renderer,
                        tex.inner.clone(),
                        tex.view.clone(),
                        None,
                        Some(&texture_config),
                        tex.extent,
                    ));
                registry.ids.insert(gpu_id.clone(), id);
                debug!("add to registry texture [no name] with id {}", id.id());
            }
        }

        // rimuove quelle che non esistono più nel texture manager
        registry.ids.retain(|gpu_id, id| {
            if !texture_cache.contains_key(gpu_id) {
                renderer.textures.remove(*id);
                debug!("remove from registry [no mame] with id {}", id.id());
                false
            } else {
                true
            }
        });
    }

    pub fn upload_textures(
        &mut self,
        gpu_context: &GpuContext,
        source: &mut impl texture_upload::TextureUploadSource,
    ) {
        self.gpu_cache
            .textures
            .upload_textures(source, &gpu_context.device, &gpu_context.queue);
    }

    fn sync_caches(&mut self, gpu_context: &GpuContext, asset_mgr: &AssetManager) {
        // Sync cleanup

        self.gpu_cache.mesh.retain(&asset_mgr.meshes);
        self.gpu_cache.material.retain(&asset_mgr.materials);
        self.gpu_cache.textures.retain(&asset_mgr.textures);

        // Sync Textures: are already on sync after upload, or fallback

        // Sync Meshes
        for (id, _value) in asset_mgr.meshes.iter() {
            self.gpu_cache
                .mesh
                .ensure(id, &asset_mgr.meshes, &self.gpu_mgr, &gpu_context.device);
        }

        // Sync Materials (crate also textures)
        for (id, _value) in asset_mgr.materials.iter() {
            self.gpu_cache.material.ensure(
                id,
                &mut self.gpu_cache.textures,
                &asset_mgr,
                &self.gpu_mgr,
                &gpu_context.device,
            );
        }
    }

    fn sync_skybox(&mut self, gpu_context: &GpuContext, asset_mgr: &AssetManager) {
        if asset_mgr.skybox.get_id() != self.skybox_mgr.get_hdr_id() {
            let hdr_texture = self.gpu_cache.textures.get_or_fallback(
                asset_mgr.skybox.get_id(),
            );
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
    pub fn prepare(&mut self, gpu_context: &GpuContext, asset_mgr: &AssetManager) {
        self.sync_skybox(gpu_context, asset_mgr);
        self.sync_caches(gpu_context, asset_mgr);
    }

    pub fn render(
        &mut self,
        gpu_context: &GpuContext,
        gpu_surface: &GpuSurface,
        asset_mgr: &AssetManager,
        world: &World,
        resources: &Resources,
        camera: &Camera,
        globals: &Globals,
        selected: Option<Entity>,
        input: &Input,
        draw_data: &imgui::DrawData,
        imgui_render: &mut ImguiRender,
    ) {
        // sync GpuCache Ids with assets Ids (meshes materials textures)
        self.prepare(gpu_context, asset_mgr);

        // update global data (uniform) to GPU
        self.update_render_globals_to_gpu(
            gpu_context,
            camera,
            globals,
            selected,
            gpu_surface.get_config(),
        );

        let frame = gpu_surface.get_frame();

        let target = frame.texture.create_view(&Default::default());
        let mut ctx = RenderContext {
            device: &gpu_context.device,
            queue: &gpu_context.queue,
            gpu_cache: &self.gpu_cache,

            gpu_mgr: &self.gpu_mgr,
            pip_mgr: &self.pipeline_mgr,
            skb_mgr: &self.skybox_mgr,
            pickobject: &self.pickobject,
            target: &target,
        };

        // Update world buffer data to gpu
        for pass in &mut self.passes {
            pass.prepare(
                asset_mgr, world, resources, camera, globals, selected, input, &mut ctx,
            );
        }

        // Render phase
        let mut encoder = gpu_context
            .device
            .create_command_encoder(&Default::default());

        for pass in &mut self.passes {
            pass.execute(&mut encoder, &mut ctx, &asset_mgr);
        }

        // Render Imgui Pass
        imgui_render.render(
            draw_data,
            &mut encoder,
            &target,
            &gpu_context.device,
            &gpu_context.queue,
        );

        gpu_context.queue.submit([encoder.finish()]);
        frame.present();
    }

    fn update_render_globals_to_gpu(
        &self,
        gpu_context: &GpuContext,
        camera: &Camera,
        globals: &Globals,
        selected: Option<Entity>,
        surface_config: &SurfaceConfiguration,
    ) {
        let entity_id = match selected {
            Some(id) => (&id).as_raw_u64(),
            None => 0,
        };

        let screen_size = [surface_config.width as f32, surface_config.height as f32];
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
