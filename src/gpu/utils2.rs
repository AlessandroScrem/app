use std::sync::{
    mpsc::{Receiver, Sender, channel},
};

pub type ReadbackId = u64;

pub struct ReadbackResult {
    pub id: ReadbackId,
    pub size: (u32, u32),
    pub bpp: u32,
    pub bytes: Vec<u8>,
}

enum ReadbackState {
    CopySubmitted,
    Mapping,
}

struct PendingReadback {
    id: ReadbackId,
    bpp: u32,
    size: (u32, u32),
    buffer: wgpu::Buffer,
}

pub struct GpuManager {
    next_id: ReadbackId,

    pending: Vec<PendingReadback>,

    completed_tx: Sender<ReadbackId>,
    completed_rx: Receiver<ReadbackId>,

    ready: Vec<ReadbackResult>,
}
impl GpuManager {
    pub fn new() -> Self {
        let (completed_tx, completed_rx) = channel();

        Self {
            next_id: 1,
            pending: Vec::new(),
            completed_tx,
            completed_rx,
            ready: Vec::new(),
        }
    }
}

impl GpuManager {
    pub fn request_readback(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        size: (u32, u32),
    ) -> ReadbackId {
        let id = self.next_id;
        self.next_id += 1;

        let bpp = super::utils::bytes_per_pixel(texture.format()).unwrap();
        let bytes_per_row =
            super::utils::align_to(size.0 * bpp, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let buffer_size = bytes_per_row * size.1;
        let buffer = create_readback_buffer(device, buffer_size as u64);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("readback encoder"),
        });
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(size.1),
                },
            },
            texture.size(),
        );

        queue.submit(Some(encoder.finish()));

        let tx = self.completed_tx.clone();

        buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                if result.is_ok() {
                    tx.send(id).ok();
                }
            });

        self.pending.push(PendingReadback {
            id,
            bpp,
            size,
            buffer,
        });

        id
    }
}

impl GpuManager {
    pub fn poll(&mut self, device: &wgpu::Device) {
        device.poll(wgpu::PollType::Poll).ok();

        while let Ok(id) = self.completed_rx.try_recv() {
            let index = self.pending.iter().position(|r| r.id == id).unwrap();

            let pending = self.pending.swap_remove(index);

            let data = pending.buffer.slice(..).get_mapped_range();
            let size = pending.size;
            let bpp = pending.bpp;

            let row = super::utils::align_to(size.0 * bpp, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
            let bytes = super::utils::unpad_image(&data, size.0, size.1, bpp, row);

            self.ready.push(ReadbackResult { id, size, bpp, bytes });

            drop(data);

            pending.buffer.unmap();
        }
    }
}

impl GpuManager {
    pub fn query_results(&mut self) -> Vec<ReadbackResult> {
        std::mem::take(&mut self.ready)
    }
}

fn create_readback_buffer(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("texture_readback"),
        size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu::caches::GpuTextureBuilder;
    use crate::gpu::static_textures;
    use crate::test_utils::get_device_and_queue;

    fn readback_poll(gpu: &mut GpuManager, device: &wgpu::Device) -> Vec<ReadbackResult> {
        use std::time::Instant;

        let start = Instant::now();
        let mut cycles = 0;
        loop {
            cycles += 1;
            gpu.poll(device);

            let results = gpu.query_results();

            if !results.is_empty() {
                println!(
                    "Readback pronto dopo {} cicli in {:?}",
                    cycles,
                    start.elapsed()
                );
                return results;
            }
            std::thread::yield_now();
        }
    }

    #[test]
    fn should_() {
        let (device, queue) = get_device_and_queue();
        let mut gpu = GpuManager::new();

        let texture = GpuTextureBuilder::from_static(&static_textures::WHITE_STATIC_TEXTURE)
            .build(device, Some(queue))
            .inner;
        let width = texture.width();
        let height = texture.height();

        gpu.request_readback(&device, &queue, &texture, (width, height));

        let results = readback_poll(&mut gpu, device);
        println!("Read {} bytes", results.len());

        for r in results.iter() {
            println!("Id {} size {:?} bpp {} bytes {:?}", r.id, r.size, r.bpp, r.bytes);
        }

        assert!(!results.is_empty());
    }
}
