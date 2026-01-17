use crate::entities::EntityRawU64;
use crate::input::Input;
use crate::ui::ImguiLayer;
use legion::{Entity, Resources, World};
use std::sync::Arc;
use wgpu::{Adapter, Device, Queue, Surface, SurfaceConfiguration};
use winit::window::Window;

use super::*;
use crate::assets::material_manager::MaterialManager;
use crate::assets::texture_manager::TextureManager;
use crate::picking::PickObject;
use crate::renderer::renderpass::*;

use crate::{Globals, prelude::*};

pub struct RenderContext<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,
    pub gpu_mgr: &'a GpuManager,
    pub pip_mgr: &'a PipelineManager,
    pub skb_mgr: &'a SkyboxManager,
    pub mat_mgr: &'a MaterialManager,
    pub mesh_mgr: &'a MeshManager,
    pub light_mgr: &'a LightManager,
    pub bbox_mgr: &'a mut BBoxManager,
    pub pickobject: &'a PickObject,
    pub target: wgpu::TextureView,
    pub imgui: &'a mut ImguiLayer,
}

pub struct GpuView<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,
    pub pickobject: &'a PickObject,
    pub gpu_mgr: &'a GpuManager,
    pub pip_mgr: &'a PipelineManager,
    pub skb_mgr: &'a SkyboxManager,
    pub mat_mgr: &'a MaterialManager,
    pub mesh_mgr: &'a MeshManager,
    pub light_mgr: &'a LightManager,
    pub bbox_mgr: &'a BBoxManager,
    pub texture_mgr: &'a TextureManager,
}

pub struct GpuDevice<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a Queue,
    pub gpu_mgr: &'a GpuManager,
    pub mat_mgr: &'a mut MaterialManager,
    pub mesh_mgr: &'a mut MeshManager,
    pub texure_mgr: &'a mut TextureManager,
    pub skb_mgr: &'a mut SkyboxManager,
}

pub struct Renderer {
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface<'static>,
    pub surface_config: SurfaceConfiguration,
    _adapter: Adapter,
    gpu_mgr: GpuManager,
    texture_mgr: TextureManager,
    pipeline_mgr: PipelineManager,
    light_mgr: LightManager,
    mesh_mgr: MeshManager,
    mat_mgr: MaterialManager,
    skybox_mgr: SkyboxManager,
    bbox_mgr: BBoxManager,

    pickobject: PickObject,
    passes: Vec<RenderPassEnum>,
}

impl Renderer {
    pub fn new(window: Arc<Window>) -> Self {
        pollster::block_on(Self::create_async(window))
    }

    async fn create_async(window: Arc<Window>) -> Self {
        let timer = std::time::Instant::now();
        info!("Initializing renderer...");
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        debug!("Device initialized in {} ms", timer.elapsed().as_millis());

        let surface = instance.create_surface(window.clone()).unwrap();
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

        let mut texture_mgr = TextureManager::new(device.clone(), queue.clone());
        let gpu_mgr = GpuManager::new(&device, size.width, size.height);
        let pipeline_mgr = PipelineManager::new(&device, &gpu_mgr, surface_config.format);
        let mat_mgr = MaterialManager::new(&device, &gpu_mgr, &mut texture_mgr);
        let light_mgr = LightManager::new(&gpu_mgr, &device, &queue);
        let mesh_mgr = MeshManager::new();
        let bbox_mgr = BBoxManager::new();
        let pickobject = PickObject::new(&device);

        let hdrpath = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/core/newport_loft.hdr");
        let skybox_mgr = SkyboxManager::new(hdrpath, &device, &queue, &gpu_mgr, &mut texture_mgr);

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
            RenderPassEnum::Imgui(ImguiPass::new()),
        ];

        Self {
            _adapter: adapter,
            device,
            queue,
            surface,
            surface_config,
            gpu_mgr,
            texture_mgr,
            pipeline_mgr,
            light_mgr,
            mesh_mgr,
            skybox_mgr,
            mat_mgr,
            bbox_mgr,
            pickobject,
            passes,
        }
    }

    pub fn get_adapter_string(&self) -> String {
        self._adapter.get_info().name
    }

    pub fn get_hdrpath(&self) -> &std::path::Path {
        &self.skybox_mgr.get_hdr_path()
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
            mat_mgr: &self.mat_mgr,
            mesh_mgr: &self.mesh_mgr,
            light_mgr: &self.light_mgr,
            bbox_mgr: &self.bbox_mgr,
            texture_mgr: &self.texture_mgr,
        }
    }

    pub fn get_gpu_mut(&mut self) -> GpuDevice<'_> {
        GpuDevice {
            device: &self.device,
            queue: &self.queue,
            gpu_mgr: &self.gpu_mgr,
            mat_mgr: &mut self.mat_mgr,
            mesh_mgr: &mut self.mesh_mgr,
            texure_mgr: &mut self.texture_mgr,
            skb_mgr: &mut self.skybox_mgr,
        }
    }

    pub fn get_mat_mgr(&self) -> &MaterialManager {
        &self.mat_mgr
    }

    pub fn get_mat_mgr_mut(&mut self) -> &mut MaterialManager {
        &mut self.mat_mgr
    }

    pub fn render(
        &mut self,
        world: &World,
        resources: &Resources,
        camera: &Camera,
        globals: &Globals,
        selected: Option<Entity>,
        input: &Input,
        imgui: &mut ImguiLayer,
    ) {
        // update global data (uniform) to GPU
        self.update_render_globals_to_gpu(camera, globals, selected);

        let frame = self.get_frame();

        let target = frame.texture.create_view(&Default::default());
        let mut ctx = RenderContext {
            device: &self.device,
            queue: &self.queue,
            gpu_mgr: &self.gpu_mgr,
            pip_mgr: &self.pipeline_mgr,
            skb_mgr: &self.skybox_mgr,
            mesh_mgr: &self.mesh_mgr,
            mat_mgr: &self.mat_mgr,
            light_mgr: &self.light_mgr,
            bbox_mgr: &mut self.bbox_mgr,
            pickobject: &self.pickobject,
            target,
            imgui,
        };

        // Update world buffer data to gpu
        for pass in &mut self.passes {
            pass.prepare(world, resources, camera, globals, selected, input, &mut ctx);
        }

        // Render phase
        let mut encoder = self.device.create_command_encoder(&Default::default());

        for pass in &mut self.passes {
            pass.execute(&mut encoder, &mut ctx);
        }

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

    pub fn resize_resources(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;

        self.surface.configure(&self.device, &self.surface_config);

        // resize gpu_manager
        self.gpu_mgr.resize_frame(&self.device, width, height);
    }
}
