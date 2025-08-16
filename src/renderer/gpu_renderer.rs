use std::sync::Arc;
use winit::window::Window;

pub struct Renderer {}
pub struct DepthTexture(pub wgpu::TextureView);

impl Renderer {
    pub async fn new(window: Arc<Window>, resources: &mut legion::Resources) {
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

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth_texture"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&Default::default());

        let gpu_resource_manager = Arc::new(
            crate::resources::gpu_manager::GPUResourceManager::new(&device, &queue),
        );
        let pipeline_manager = crate::renderer::pipeline_manager::PipelineManager::new();

        let material_manager = crate::assets::material_manager::MaterialManager::new(
            Arc::new(device.clone()),
            Arc::new(queue.clone()),
            gpu_resource_manager.clone(),
        );

        resources.insert(surface_config);
        resources.insert(material_manager);
        resources.insert(pipeline_manager);
        resources.insert(gpu_resource_manager);
        resources.insert(queue);
        resources.insert(surface);
        resources.insert(DepthTexture(depth_view));
        resources.insert(device);
    }
}
