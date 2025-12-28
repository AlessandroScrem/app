use std::sync::Arc;
use winit::window::Window;

use crate::assets::material_manager;
use crate::assets::mesh_manager::MeshManager;
use crate::assets::texture_manager;
use crate::picking::PickObject;
use crate::prelude::*;
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

        debug!("Device initialized in {} ms", timer.elapsed().as_millis());

        let gpu_manager = gpu_manager::GPUResourceManager::new(&device, size.width, size.height);
        let pipeline_manager = pipeline_manager::PipelineManager::new(
            &device,
            &gpu_manager,
            surface_config.format,
        );
        debug!(
            "Pipeline manager initialized in {} ms",
            timer.elapsed().as_millis()
        );

        let mut texture_manager =
            texture_manager::TextureManager::new(Arc::new(device.clone()), Arc::new(queue.clone()));
        debug!(
            "Texture manager initialized in {} ms",
            timer.elapsed().as_millis()
        );

        let material_manager = material_manager::MaterialManager::new(
            &device,
            &gpu_manager,
            &mut texture_manager,
        );
        debug!(
            "Material manager initialized in {} ms",
            timer.elapsed().as_millis()
        );

        let light_manager = light_manager::LightManager::new(
            &gpu_manager,
            Arc::new(device.clone()),
            Arc::new(queue.clone()),
        );
        debug!(
            "Light manager initialized in {} ms",
            timer.elapsed().as_millis()
        );

        let hdrpath = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/core/newport_loft.hdr");
        let skybox_manager = skybox_manager::SkyboxManager::new(
            hdrpath,
            &device,
            &queue,
            &gpu_manager,
            &mut texture_manager,
        );
        debug!(
            "Skybox manager initialized in {} ms",
            timer.elapsed().as_millis()
        );

        let bbox_manager = bbox_manager::BBoxManager::new();
        let mesh_manager = MeshManager::new();
        let pickobject = PickObject::new(&device);

        resources.insert(adapter);
        resources.insert(device);
        resources.insert(queue);
        resources.insert(surface);
        resources.insert(surface_config);
        resources.insert(gpu_manager);
        resources.insert(pipeline_manager);
        resources.insert(material_manager);
        resources.insert(mesh_manager);
        resources.insert(texture_manager);
        resources.insert(light_manager);
        resources.insert(skybox_manager);
        resources.insert(bbox_manager);
        resources.insert(pickobject);
    }
}
