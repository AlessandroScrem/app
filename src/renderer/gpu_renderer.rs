use std::sync::Arc;
use winit::window::Window;

use crate::assets::material_manager::MaterialManager;
use crate::assets::mesh_manager::MeshManager;
use crate::assets::texture_manager::TextureManager;
use crate::picking::PickObject;
use crate::prelude::*;
use crate::renderer::light_manager::LightManager;
use crate::renderer::pipeline_manager::PipelineManager;
use crate::renderer::skybox_manager::SkyboxManager;
use crate::renderer::*;

pub struct Renderer {}

impl Renderer {
    pub fn init(window: Arc<Window>, resources: &mut legion::Resources) {
        pollster::block_on(Self::init_async(window, resources));
    }

    async fn init_async(window: Arc<Window>, resources: &mut legion::Resources) {
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

        info!("Surface config format is {:?}", surface_config.format);
        
        let mut texture_manager = TextureManager::new(device.clone(), queue.clone());
        let gpu_manager = GpuManager::new(&device, size.width, size.height);
        let pipeline_manager = PipelineManager::new(&device, &gpu_manager, surface_config.format);
        let material_manager = MaterialManager::new(&device, &gpu_manager, &mut texture_manager);
        let light_manager = LightManager::new(&gpu_manager, &device, &queue);
        let mesh_manager = MeshManager::new();
        let bbox_manager = bbox_manager::BBoxManager::new();
        let pickobject = PickObject::new(&device);

        let hdrpath = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/core/newport_loft.hdr");
        let skybox_manager =
            SkyboxManager::new(hdrpath, &device, &queue, &gpu_manager, &mut texture_manager);
        debug!(
            "Skybox manager initialized in {} ms",
            timer.elapsed().as_millis()
        );

        let imgui = ui::ImguiState::new(&window, &device, &queue, surface_format);


        resources.insert(adapter);
        resources.insert(device);
        resources.insert(queue);
        resources.insert(surface);
        resources.insert(surface_config);
        resources.insert(texture_manager);
        resources.insert(gpu_manager);
        resources.insert(pipeline_manager);
        resources.insert(material_manager);
        resources.insert(light_manager);
        resources.insert(mesh_manager);
        resources.insert(bbox_manager);
        resources.insert(pickobject);
        resources.insert(skybox_manager);
        resources.insert(imgui);
    }
}
