use crate::assets::asset_manager::AssetManager;
use crate::entities::EntityRawU64;
use crate::input::Input;
use imgui_wgpu::RendererConfig;
use legion::{Entity, Resources, World};
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::{Adapter, Device, Queue, Surface, SurfaceConfiguration};
use winit::window::Window;

use super::*;
use crate::picking::PickObject;
use crate::renderer::renderpass::*;

use crate::{Globals, prelude::*};

pub struct RenderContext<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,
    pub gpu_cache: &'a GpuCache,

    pub gpu_mgr: &'a GpuManager,
    pub pip_mgr: &'a PipelineManager,
    pub skb_mgr: &'a SkyboxManager,
    pub light_mgr: &'a LightManager,
    pub bbox_mgr: &'a mut BBoxManager,
    pub pickobject: &'a PickObject,
    pub target: &'a wgpu::TextureView,
}

pub struct GpuView<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,
    pub pickobject: &'a PickObject,
    pub gpu_mgr: &'a GpuManager,
    pub pip_mgr: &'a PipelineManager,
    pub skb_mgr: &'a SkyboxManager,
    pub light_mgr: &'a LightManager,
    pub bbox_mgr: &'a BBoxManager,
}

pub struct GpuDevice<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a Queue,
    pub gpu_mgr: &'a GpuManager,
    pub skb_mgr: &'a mut SkyboxManager,
}

// registro imgui separato
pub struct ImGuiTextureRegistry {
    pub ids: HashMap<TextureId, imgui::TextureId>,
}

impl ImGuiTextureRegistry {
    pub fn new() -> Self {
        Self {
            ids: HashMap::new(),
        }
    }
}
pub struct ImguiRender {
    renderer: imgui_wgpu::Renderer,
    registry: ImGuiTextureRegistry,
}

impl ImguiRender {
    fn new(
        device: &Device,
        queue: &Queue,
        context: &mut imgui::Context,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        let renderer_config = RendererConfig {
            texture_format,
            ..Default::default()
        };
        let renderer = imgui_wgpu::Renderer::new(context, &device, &queue, renderer_config);
        let registry = ImGuiTextureRegistry::new();

        Self { renderer, registry }
    }
    pub fn render(
        &mut self,
        draw_data: &imgui::DrawData,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        device: &Device,
        queue: &Queue,
    ) {
        let frame_view = target;

        // Render pass
        let mut pass = {
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ImGui Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: frame_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // non cancellare la scena
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            })
        };

        match self.renderer.render(draw_data, queue, device, &mut pass) {
            Ok(()) => {} 
            Err(e) => {
                error!("Imgui Render failed: {:?}", e);
            }
        }
    }
}
#[derive(Default)]
pub struct GpuCache {
    pub mesh: GpuMeshCache,
    pub material: GpuMaterialCache,
    pub textures: GpuTextureCache,
}

pub struct Renderer {
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface<'static>,
    pub surface_config: SurfaceConfiguration,
    _adapter: Adapter,
    gpu_mgr: GpuManager,
    pipeline_mgr: PipelineManager,
    light_mgr: LightManager,
    skybox_mgr: SkyboxManager,
    bbox_mgr: BBoxManager,
    imgui_render: ImguiRender,

    pickobject: PickObject,
    passes: Vec<RenderPassEnum>,

    gpu_cache: GpuCache,
}

impl Renderer {
    pub fn new(
        window: Arc<Window>,
        imgui_ctx: &mut imgui::Context,
        asset_mgr: &mut AssetManager,
    ) -> Self {
        pollster::block_on(Self::create_async(window, imgui_ctx, asset_mgr))
    }

    async fn create_async(
        window: Arc<Window>,
        imgui_ctx: &mut imgui::Context,
        asset_mgr: &mut AssetManager,
    ) -> Self {
        let timer = std::time::Instant::now();
        info!("Initializing renderer...");
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("unable to  crate adapter");
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .expect("unable to create device");

        debug!("Device initialized in {} ms", timer.elapsed().as_millis());

        let surface = instance
            .create_surface(window.clone())
            .expect("unable to create Surface");
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats[0];

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let gpu_mgr = GpuManager::new(&device, size.width, size.height);
        let pipeline_mgr = PipelineManager::new(&device, &gpu_mgr, surface_config.format);
        let light_mgr = LightManager::new(&gpu_mgr, &device, &queue);
        let bbox_mgr = BBoxManager::new();
        let pickobject = PickObject::new(&device);

        // Skybox initialization
        let mut texture_cache = GpuTextureCache::default();
        let hdr_id = asset_mgr.skybox.get_id();
        let hdr = texture_cache.get_or_create(hdr_id, &asset_mgr.textures, &device, &queue);
        let skybox_mgr = SkyboxManager::new(hdr_id, hdr, &device, &queue, &gpu_mgr);
        // -----

        let imgui_render = ImguiRender::new(&device, &queue, imgui_ctx, surface_config.format);

        info!(
            "Renderer Created: Surface config format is {:?}",
            surface_config.format
        );

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
            ..Default::default()
        };

        Self {
            _adapter: adapter,
            device,
            queue,
            surface,
            surface_config,
            gpu_mgr,
            pipeline_mgr,
            light_mgr,
            skybox_mgr,
            bbox_mgr,
            pickobject,
            imgui_render,
            passes,
            gpu_cache,
        }
    }

    pub fn get_adapter_string(&self) -> String {
        self._adapter.get_info().name
    }

    pub fn get_hdr_id(&self) -> TextureId {
        self.skybox_mgr.get_hdr_id()
    }

    pub fn get_hdr_imgui_id(&self) -> Option<&imgui::TextureId> {
        let hdr_id = self.get_hdr_id();
        let registry = self.get_texture_registry();
        registry.ids.get(&hdr_id)
    }

    pub fn get_hovered(&mut self) -> Option<Entity> {
        self.pickobject.poll_readback(&self.device)
    }

    pub fn get_encoder(&self) -> wgpu::CommandEncoder {
        self.device.create_command_encoder(&Default::default())
    }

    pub fn get_frame(&self) -> wgpu::SurfaceTexture {
        self.surface
            .get_current_texture()
            .expect("Failed to get current texture")
    }

    pub fn get_gpu_view(&mut self) -> GpuView<'_> {
        GpuView {
            device: &self.device,
            queue: &self.queue,
            pickobject: &self.pickobject,
            gpu_mgr: &self.gpu_mgr,
            pip_mgr: &self.pipeline_mgr,
            skb_mgr: &self.skybox_mgr,
            light_mgr: &self.light_mgr,
            bbox_mgr: &self.bbox_mgr,
        }
    }

    pub fn get_gpu_mut(&mut self) -> GpuDevice<'_> {
        GpuDevice {
            device: &self.device,
            queue: &self.queue,
            gpu_mgr: &self.gpu_mgr,
            skb_mgr: &mut self.skybox_mgr,
        }
    }

    pub fn get_texture_registry(&self) -> &ImGuiTextureRegistry {
        &self.imgui_render.registry
    }

    pub fn sync_imgui_texture(&mut self) {
        let registry = &mut self.imgui_render.registry;
        let renderer = &mut self.imgui_render.renderer;
        let texture_cache = &self.gpu_cache.textures;
        let device = &self.device;

        // record new textures
        use imgui_wgpu::RawTextureConfig;
        for (gpu_id, tex) in texture_cache.iter() {
            if !registry.ids.contains_key(gpu_id) {
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
                        tex.texture.inner.clone(),
                        tex.texture.view.clone(),
                        None,
                        Some(&texture_config),
                        tex.texture.extent,
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

    pub fn prepare(&mut self, asset_mgr: &AssetManager) {
        // skybox
        if asset_mgr.skybox.get_id() != self.skybox_mgr.get_hdr_id() {
            let hdr_texture = self.gpu_cache.textures.get_or_create(
                asset_mgr.skybox.get_id(),
                &asset_mgr.textures,
                &self.device,
                &self.queue,
            );
            self.skybox_mgr.update_skybox(
                asset_mgr.skybox.get_id(),
                hdr_texture,
                &self.device,
                &self.queue,
                &self.gpu_mgr,
            );
        }
        // meshes
        let mesh_cache = &mut self.gpu_cache.mesh;
        for (id, _desc) in asset_mgr.meshes.iter() {
            mesh_cache.ensure(id, &asset_mgr.meshes, &self.gpu_mgr, &self.device);
        }

        // materials
        let material_cache = &mut self.gpu_cache.material;
        for (id, _desc) in asset_mgr.materials.iter() {
            material_cache.ensure(
                id,
                &mut self.gpu_cache.textures,
                &asset_mgr,
                &self.gpu_mgr,
                &self.device,
                &self.queue,
            );
        }

        //textures
        let texture_cache = &mut self.gpu_cache.textures;
        for (id, _desc) in asset_mgr.textures.iter() {
            texture_cache.ensure(id, &asset_mgr.textures, &self.device, &self.queue);
        }
    }

    pub fn render(
        &mut self,
        asset_mgr: &AssetManager,
        world: &World,
        resources: &Resources,
        camera: &Camera,
        globals: &Globals,
        selected: Option<Entity>,
        input: &Input,
        draw_data: &imgui::DrawData,
    ) {
        self.prepare(asset_mgr);

        // update global data (uniform) to GPU
        self.update_render_globals_to_gpu(camera, globals, selected);

        let frame = self.get_frame();

        let target = frame.texture.create_view(&Default::default());
        let mut ctx = RenderContext {
            device: &self.device,
            queue: &self.queue,
            gpu_cache: &self.gpu_cache,

            gpu_mgr: &self.gpu_mgr,
            pip_mgr: &self.pipeline_mgr,
            skb_mgr: &self.skybox_mgr,
            light_mgr: &self.light_mgr,
            bbox_mgr: &mut self.bbox_mgr,
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
        let mut encoder = self.device.create_command_encoder(&Default::default());

        for pass in &mut self.passes {
            pass.execute(&mut encoder, &mut ctx);
        }

        // Render Imgui Pass
        self.imgui_render
            .render(draw_data, &mut encoder, &target, &self.device, &self.queue);

        self.queue.submit([encoder.finish()]);
        frame.present();
    }

    fn update_render_globals_to_gpu(
        &self,
        camera: &Camera,
        globals: &Globals,
        selected: Option<Entity>,
    ) {
        let entity_id = match selected {
            Some(id) => (&id).as_raw_u64(),
            None => 0,
        };

        let screen_size = [
            self.surface_config.width as f32,
            self.surface_config.height as f32,
        ];
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

        self.queue.write_buffer(
            &self.gpu_mgr.camera_uniform_buffer,
            0,
            bytemuck::bytes_of(&updated_camera_uniform),
        );
        self.queue.write_buffer(
            &self.gpu_mgr.globals_uniform_buffer,
            0,
            bytemuck::bytes_of(&updated_globals_uniform),
        );
    }

    pub fn resize_frame(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;

        self.surface.configure(&self.device, &self.surface_config);

        // resize gpu_manager
        self.gpu_mgr.resize_frame(&self.device, width, height);
    }
}
