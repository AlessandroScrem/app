#![allow(unused)]

use std::{
    ops::Not,
    sync::{
        Arc,
        mpsc::{Receiver, Sender, channel},
    },
};

use crate::gpu::utils::unpad_image;

pub struct GpuManager {
    result_tx: Sender<ReadbackResult>,
    result_rx: Receiver<ReadbackResult>,
}
impl GpuManager {
    pub fn new() -> Self {
        let (result_tx, result_rx) = channel();

        Self {
            result_tx,
            result_rx,
        }
    }
}

impl GpuManager {
    pub fn request_readback(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        origin: (u32, u32),
        size: (u32, u32),
        decoder: Box<dyn ReadbackDecoder>,
    ) {
        let bpp = super::utils::bytes_per_pixel(texture.format()).unwrap();
        let bytes_per_row =
            super::utils::align_to(size.0 * bpp, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let buffer_size = bytes_per_row * size.1;
        let buffer = Arc::new(create_readback_buffer(device, buffer_size as u64));

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("readback encoder"),
        });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: origin.0,
                    y: origin.1,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(size.1),
                },
            },
            wgpu::Extent3d {
                depth_or_array_layers: 1,
                width: size.0,
                height: size.1,
            },
        );

        queue.submit(Some(encoder.finish()));

        let tx = self.result_tx.clone();
        let buffer_clone = buffer.clone();

        buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |status| {
                if status.is_err() {
                    return;
                }

                let data = buffer_clone.slice(..).get_mapped_range();
                let bytes = unpad_image(&data, size.0, size.1, bpp, bytes_per_row);

                let result = decoder.decode(bytes, size);

                tx.send(result).ok();

                drop(data);
                buffer_clone.unmap();
            });
    }
}

impl GpuManager {
    pub fn poll(&mut self, device: &wgpu::Device) {
        device.poll(wgpu::PollType::Poll).ok();
    }
}

impl GpuManager {
    pub fn query_results(&mut self) -> Vec<ReadbackResult> {
        let mut results = Vec::new();

        while let Ok(result) = self.result_rx.try_recv() {
            results.push(result);
        }

        results
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

pub enum ReadbackResult {
    Selection(Vec<u64>),
    Pick(Option<u64>),
    RawData(Vec<u8>),
}

pub trait ReadbackDecoder: Send {
    fn decode(self: Box<Self>, bytes: Vec<u8>, size: (u32, u32)) -> ReadbackResult;
}

pub struct RawData;
impl ReadbackDecoder for RawData {
    fn decode(self: Box<Self>, bytes: Vec<u8>, _: (u32, u32)) -> ReadbackResult {
        ReadbackResult::RawData(bytes)
    }
}

pub struct Pick;
impl ReadbackDecoder for Pick {
    fn decode(self: Box<Self>, bytes: Vec<u8>, _: (u32, u32)) -> ReadbackResult {
        // let id = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let id = bytes.len();

        ReadbackResult::Pick(if id == 0 {
            None
        } else {
            Some(id as u64)
            // Some(Entity::from_raw_u64(id as u64))
        })
    }
}

pub struct Selection;
impl ReadbackDecoder for Selection {
    fn decode(self: Box<Self>, bytes: Vec<u8>, _: (u32, u32)) -> ReadbackResult {
        let ids = bytes.chunks_exact(4).map(|pixel| pixel[0] as u64).filter(|&id| id != 0).collect();
        ReadbackResult::Selection(ids)
    }
}

#[cfg(test)]
mod tests {
    use log::error;

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
    fn should_read_rawdata() {
        let (device, queue) = get_device_and_queue();
        let mut gpu = GpuManager::new();

        let gpu_texture =
            GpuTextureBuilder::from_static(&static_textures::LIGHTBULB_STATIC_TEXTURE)
                .build(device, Some(queue));
        let texture = gpu_texture.inner;

        println!("Texture format: {:?}", texture.format());

        let origin = (0, 0);
        let size = (texture.width(), texture.height());

        gpu.request_readback(&device, &queue, &texture, origin, size, Box::new(RawData));

        let results = readback_poll(&mut gpu, device);
        println!("Read {} results", results.len());

        let bpp = gpu_texture.format.pixel_size();
        let raw_result_size = size.0 * size.1 * bpp;

        for r in results.iter() {
            match r {
                ReadbackResult::RawData(vec) => {
                    assert_eq!(raw_result_size as usize, vec.len());
                    // println!("Bytes {:?}", vec);
                }
               _ => panic!("unexpected"),
            }
        }
        assert!(!results.is_empty());
    }

    #[test]
    fn should_read_pick() {
        let (device, queue) = get_device_and_queue();
        let mut gpu = GpuManager::new();

        let gpu_texture =
            GpuTextureBuilder::from_static(&static_textures::LIGHTBULB_STATIC_TEXTURE)
                .build(device, Some(queue));
        let texture = gpu_texture.inner;

        println!("Texture format: {:?}", texture.format());

        let origin = (15, 15);
        let size = (1, 1);

        gpu.request_readback(&device, &queue, &texture, origin, size, Box::new(Pick));

        let results = readback_poll(&mut gpu, device);
        println!("Read {} results", results.len());

        let bpp = gpu_texture.format.pixel_size();
        let raw_result_size = size.0 * size.1 * bpp;

        for r in results.iter() {
            match r {
                ReadbackResult::Pick(id) => {
                    assert_eq!(*id , Some(4));
                    println!("id {:?}", id);
                }
                _ => panic!("unexpected"),
            }
        }
        assert!(!results.is_empty());
    }

    #[test]
    fn should_read_selection() {
        let (device, queue) = get_device_and_queue();
        let mut gpu = GpuManager::new();

        let gpu_texture =
            GpuTextureBuilder::from_static(&static_textures::LIGHTBULB_STATIC_TEXTURE)
                .build(device, Some(queue));
        let texture = gpu_texture.inner;

        println!("Texture format: {:?}", texture.format());

        let origin = (0, 0);
        let size = (10, 8);

        gpu.request_readback(&device, &queue, &texture, origin, size, Box::new(Selection));

        let results = readback_poll(&mut gpu, device);
        println!("Read {} results", results.len());

        let bpp = gpu_texture.format.pixel_size();
        let raw_result_size = size.0 * size.1 * bpp;

        for r in results.iter() {
            match r {
                ReadbackResult::Selection(ids) => {
                    assert!(!ids.is_empty());
                    println!("id count  {:?}", ids.len() );
                }
                _ => panic!("unexpected"),
            }
        }
        assert!(!results.is_empty());
    }
}
