use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use winit::window::Window;

use crate::renderer::hdr_frame::HdrFrame;
use crate::renderer::hdr_frame::IDTexture;
use crate::renderer::light_manager;
use crate::renderer::skybox_manager;

pub struct Renderer {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickState {
    Idle,    // pronto per una nuova copia
    Copying, // in corso: la GPU sta scrivendo nel buffer
    Mapped,  // mappato e pronto da leggere
}

pub struct PickBuffer {
    pub staging: Arc<wgpu::Buffer>,
    pub last_id: Arc<AtomicU64>,
    pub ready: Arc<AtomicBool>,
    pub state: Arc<std::sync::Mutex<PickState>>,
}

impl PickBuffer {
    pub fn new(device: &wgpu::Device) -> Self {
        let staging = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("staging Readback Pixel"),
            size: 256,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        }));
        let last_id = Arc::new(AtomicU64::new(0));
        let ready = Arc::new(AtomicBool::new(true));
        Self {
            staging,
            last_id,
            ready,
            state: Arc::new(std::sync::Mutex::new(PickState::Idle)),
        }
    }

    pub fn read_id(&self) {
        use std::sync::atomic::Ordering;

        let mut state = self.state.lock().unwrap();
        if *state != PickState::Idle {
            // Evita doppio map se non ancora completato
            return;
        }

        *state = PickState::Copying;
        self.ready.store(false, Ordering::Relaxed);

        let staging = Arc::clone(&self.staging);
        let last_id = Arc::clone(&self.last_id);
        let staging_clone = Arc::clone(&staging);
        let ready = Arc::clone(&self.ready);
        let state_arc = Arc::clone(&self.state);
        // let timer = std::time::Instant::now();

        // println!("Mapping buffer flag is: {}", ready.load(Ordering::Relaxed));
        // Map_async direttamente sul buffer
        staging_clone
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |res| {
                let mut state = state_arc.lock().unwrap();
                if let Ok(()) = res {
                    // Prendi la slice solo qui dentro, non prima
                    let data = staging.slice(..).get_mapped_range();

                    if data.len() >= 8 {
                        let r = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                        let g = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
                        let id = ((g as u64) << 32) | (r as u64);
                        last_id.store(id, Ordering::Relaxed);
                        // println!("✅ ID {} letto in {:?}", id, timer.elapsed());
                    } else {
                        eprintln!("❌ buffer troppo piccolo per leggere ID");
                    }

                    drop(data);
                    staging.unmap();
                    ready.store(true, Ordering::Relaxed);
                    *state = PickState::Mapped;
                } else {
                    eprintln!("❌ map_async fallita");
                    *state = PickState::Idle;
                    ready.store(true, Ordering::Relaxed);
                }
            });
    }

    /// Ritorna l'ultimo ID valido (se pronto)
    pub fn get_id_if_ready(&self) -> Option<u64> {
        use std::sync::atomic::Ordering;

        if self.ready.load(Ordering::Relaxed) {
            let mut state = self.state.lock().unwrap();
            if *state == PickState::Mapped {
                *state = PickState::Idle; // pronto per un nuovo ciclo
            }
            Some(self.last_id.load(Ordering::Relaxed))
        } else {
            None
        }
    }
}

pub struct Hovered(pub u64);

#[derive(Default, Clone, Copy)]
pub struct PickPoint {
    pub x: u32,
    pub y: u32,
}

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

        let skybox_manager = skybox_manager::SkyboxManager::new(
            &device,
            &queue,
            &gpu_resource_manager,
            &mut texture_manager,
        );
        println!(
            "Skybox manager initialized in {} ms",
            timer.elapsed().as_millis()
        );

        println!("Surface config format is {:?}", surface_config.format);

        let hdr_frame = HdrFrame::new(&device, &gpu_resource_manager, size);
        let entity_id_texture = IDTexture::new(&device, &gpu_resource_manager, size);
        let ibl_bind_group = Ibl::new(
            &device,
            &gpu_resource_manager,
            &skybox_manager,
            &light_manager,
        );

        let pickbuffer = PickBuffer::new(&device);
        
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
        resources.insert(entity_id_texture);
        resources.insert(ibl_bind_group);
        resources.insert(adapter);
        resources.insert(pickbuffer);
        resources.insert(PickPoint::default());
        resources.insert(Hovered(0));
    }
}
