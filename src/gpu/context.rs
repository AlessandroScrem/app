use super::*;

use wgpu::*;

pub struct GpuContext {
    instance: Instance,
    adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
}

impl Default for GpuContext {
    fn default() -> Self {
        let timer = std::time::Instant::now();
        info!("Initializing Gpu...");

        let instance = wgpu::Instance::default();
        let adapter =
            pollster::block_on(instance.request_adapter(&RequestAdapterOptions::default()))
                .expect("unable to  crate adapter");

        let (device, queue) =
            pollster::block_on(adapter.request_device(&DeviceDescriptor::default()))
                .expect("unable to create device");

        debug!("Device initialized in {} ms", timer.elapsed().as_millis());
        Self {
            instance,
            adapter,
            device,
            queue,
        }
    }
}
impl GpuContext {
    pub fn adapter(&self) -> &Adapter {
        &self.adapter
    }
    pub fn instance(&self) -> &Instance {
        &self.instance
    }
    pub fn get_adapter_string(&self) -> String {
        self.adapter.get_info().name
    }

    pub fn create_encoder(&mut self) -> CommandEncoder {
        self.device.create_command_encoder(&Default::default())
    }
}
