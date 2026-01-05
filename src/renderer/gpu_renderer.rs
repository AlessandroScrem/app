use legion::Entity;
use std::sync::Arc;
use wgpu::{Adapter, Device, Queue, Surface, SurfaceConfiguration};
use winit::window::Window;

use crate::assets::material_manager::{MaterialId, MaterialManager};
use crate::assets::mesh_manager::MeshManager;
use crate::assets::texture_manager::TextureManager;
use crate::picking::PickObject;
use crate::renderer::bbox_manager::{BBoxManager, BBoxVertexData};
use crate::renderer::light_manager::LightManager;
use crate::renderer::pipeline_manager::PipelineManager;
use crate::renderer::skybox_manager::SkyboxManager;
use crate::renderer::uniform::{MaterialUniform, ModelUniform};
use crate::renderer::*;
use crate::{Globals, prelude::*};

pub struct GpuView<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,
    pub pickobject: &'a PickObject,
    pub gpu_mgr: &'a GpuManager,
    pub pip_mgr: &'a PipelineManager,
    pub skb_mgr: &'a SkyboxManager,
    pub mat_mgr: &'a mut MaterialManager,
    pub mesh_mgr: &'a mut MeshManager,
    pub light_mgr: &'a mut LightManager,
    pub bbox_mgr: &'a mut BBoxManager,
    pub texture_mgr: &'a mut TextureManager,
}
pub struct GpuMeshFrame {
    pub mesh_handle: usize,
    pub material_id: MaterialId,
    pub model: ModelUniform,
}

pub struct GpuBoxFrame {
    pub vertices: BBoxVertexData,
    pub entity: Entity,
}
pub struct RenderFrame {
    pub meshes: Vec<GpuMeshFrame>,
    pub lights: Vec<LightUniform>,
    pub bboxes: Vec<GpuBoxFrame>,
    pub globals: Globals,
    pub camera: Camera,
    pub entity_id: u64,
}

pub struct Renderer {
    pub device: Device,
    pub queue: Queue,
    adapter: Adapter,
    pub surface: Surface<'static>,
    pub surface_config: SurfaceConfiguration,
    gpu_mgr: GpuManager,
    texture_mgr: TextureManager,
    pipeline_mgr: PipelineManager,
    light_mgr: LightManager,
    mesh_mgr: MeshManager,
    mat_mgr: MaterialManager,
    skybox_mgr: SkyboxManager,
    bbox_mgr: BBoxManager,

    pub pickobject: PickObject,
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
        
        info!("Renderer Created: Surface config format is {:?}", surface_config.format);

        Self {
            adapter,
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
        }
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
            mat_mgr: &mut self.mat_mgr,
            mesh_mgr: &mut self.mesh_mgr,
            light_mgr: &mut self.light_mgr,
            bbox_mgr: &mut self.bbox_mgr,
            texture_mgr: &mut self.texture_mgr,
        }
    }

    pub fn get_mat_mgr(&self) ->&MaterialManager {
        &self.mat_mgr
    }

    pub fn prepare(&mut self, render_frmae: &RenderFrame) {
        self.update_globals(render_frmae);
        self.update_lights(render_frmae);
        self.update_meshes(render_frmae);
        self.update_bbox(render_frmae);
    }

    fn update_lights(&self, render_frmae: &RenderFrame) {
        for light in render_frmae.lights.iter() {
            self.queue.write_buffer(
                &self.gpu_mgr.light_uniform_buffer,
                0,
                bytemuck::bytes_of(light),
            );
        }
    }

    fn update_bbox(&mut self, render_frmae: &RenderFrame) {
        for bbox in render_frmae.bboxes.iter() {
            self.queue.write_buffer(
                &self.bbox_mgr.get_or_create(&self.device, bbox.entity),
                0,
                bytemuck::cast_slice(&bbox.vertices.as_slice()),
            );
        }
    }

    fn update_meshes(&self, render_frmae: &RenderFrame) {
        for mesh in render_frmae.meshes.iter() {
            // Material Uniform
            let material = self.mat_mgr.get(&mesh.material_id);
            let updated_uniforms = MaterialUniform::from(&material.material_pbr);
            self.queue.write_buffer(
                &material.uniform_buffer,
                0,
                bytemuck::bytes_of(&updated_uniforms),
            );

            // Model Uniform
            self.queue.write_buffer(
                &self.mesh_mgr.get_model_uniform(mesh.mesh_handle),
                0,
                bytemuck::bytes_of(&mesh.model),
            );
        }
    }

    fn update_globals(&self, render_frmae: &RenderFrame) {
        let camera = &render_frmae.camera;
        let globals = &render_frmae.globals;
        let entity_id = render_frmae.entity_id;

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
