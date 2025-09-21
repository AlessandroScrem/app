use std::sync::Arc;
use winit::window::Window;

use crate::renderer::hdr_frame::HdrFrame;
use crate::renderer::light_manager;
use crate::renderer::skybox_manager;

pub struct Renderer {}
pub struct DepthTexture(pub wgpu::TextureView);

pub struct Ibl {
    pub ibl_bind_group: wgpu::BindGroup,
}

impl Ibl {
    pub fn new(
        device: &wgpu::Device,
        gpu_resource_manager: &crate::renderer::gpu_manager::GPUResourceManager,
        skybox_manager: &skybox_manager::SkyboxManager,
        light_manager: &light_manager::LightManager,
    ) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let ibl_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Ibl Bind Group"),
            layout: &gpu_resource_manager
                .get_layout(crate::renderer::gpu_manager::LayoutKind::LightIbl),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &light_manager.light_uniform_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(skybox_manager.get_irradiance()),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(skybox_manager.get_prefilter()),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(skybox_manager.get_brdf_lut()),
                },
            ],
        });

        Self { ibl_bind_group }
    }
}

impl Renderer {
    pub async fn new(window: Arc<Window>, resources: &mut legion::Resources) {
        let timer = std::time::Instant::now();
        println!("Initializing renderer...");
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

        println!("Device initialized in {} ms", timer.elapsed().as_millis());

        let gpu_resource_manager = Arc::new(crate::renderer::gpu_manager::GPUResourceManager::new(
            &device,
        ));
        let pipeline_manager = crate::renderer::pipeline_manager::PipelineManager::new(
            &device,
            &gpu_resource_manager,
            surface_config.format,
        );
        println!(
            "Pipeline manager initialized in {} ms",
            timer.elapsed().as_millis()
        );

        let material_manager = crate::assets::material_manager::MaterialManager::new(
            Arc::new(device.clone()),
            gpu_resource_manager.clone(),
        );
        println!(
            "Material manager initialized in {} ms",
            timer.elapsed().as_millis()
        );

        let mut texture_manager = crate::assets::texture_manager::TextureManager::new(
            Arc::new(device.clone()),
            Arc::new(queue.clone()),
        );
        println!(
            "Texture manager initialized in {} ms",
            timer.elapsed().as_millis()
        );

        let light_manager = light_manager::LightManager::new(
            &gpu_resource_manager,
            Arc::new(device.clone()),
            Arc::new(queue.clone()),
        );
        println!(
            "Light manager initialized in {} ms",
            timer.elapsed().as_millis()
        );

        let skybox_manager =
            skybox_manager::SkyboxManager::new(&device, &queue, &gpu_resource_manager, &mut texture_manager);
        println!(
            "Skybox manager initialized in {} ms",
            timer.elapsed().as_millis()
        );

        println!("Surface config format is {:?}", surface_config.format);

        let hdr_frame = HdrFrame::new(&device, &gpu_resource_manager, size);
        let ibl_bind_group = Ibl::new(
            &device,
            &gpu_resource_manager,
            &skybox_manager,
            &light_manager,
        );

        resources.insert(device);
        resources.insert(queue);
        resources.insert(surface);
        resources.insert(surface_config);
        resources.insert(gpu_resource_manager);
        resources.insert(pipeline_manager);
        resources.insert(material_manager);
        resources.insert(texture_manager);
        resources.insert(light_manager);
        resources.insert(skybox_manager);
        resources.insert(DepthTexture(depth_view));
        resources.insert(hdr_frame);
        resources.insert(ibl_bind_group);
    }
}
