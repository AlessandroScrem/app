use std::sync::Arc;
use winit::window::Window;

pub struct Renderer {}

impl Renderer {
    pub async fn new(window: Arc<Window>, resources: &mut legion::Resources) {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
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

        let gpu_resource_manager = crate::resources::gpu_manager::GPUResourceManager::new(&device);
        let pipeline_manager = crate::renderer::pipeline_manager::PipelineManager::new();

        resources.insert(surface_config);
        resources.insert(pipeline_manager);
        resources.insert(gpu_resource_manager);
        resources.insert(queue);
        resources.insert(surface);
        resources.insert(device);
    }
}
