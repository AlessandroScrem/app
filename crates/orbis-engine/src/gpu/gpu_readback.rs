use crate::gpu::utils;
use std::sync::{
    Arc,
    mpsc::{Receiver, channel},
};

pub trait ReadbackProvider {
    fn request_readback(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        origin: (u32, u32),
        size: (u32, u32),
    ) -> ReadbackHandle;

    #[allow(unused)]
    fn poll(device: &wgpu::Device) {
        device.poll(wgpu::PollType::Poll).ok();
    }
}

#[derive(Default)]
pub struct GpuReadback;

pub struct ReadbackResult {
    pub bytes: Vec<u8>,
    #[allow(unused)]
    pub size: (u32, u32),
}

pub struct ReadbackHandle {
    receiver: Receiver<ReadbackResult>,
}

impl ReadbackHandle {
    pub fn try_recv(&self) -> Option<ReadbackResult> {
        self.receiver.try_recv().ok()
    }
}

impl ReadbackProvider for GpuReadback {
    fn request_readback(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        origin: (u32, u32),
        size: (u32, u32),
    ) -> ReadbackHandle {
        assert!(size.0 > 0 && size.1 > 0, "Error: invalid size: must be > 0");
        assert!(
            origin.0 + size.0 <= texture.width() && origin.1 + size.1 <= texture.height(),
            "Error: invalid coords"
        );

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
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
        );

        queue.submit(Some(encoder.finish()));

        let (tx, rx) = channel();

        let buffer_clone = buffer.clone();

        buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |status| {
                if status.is_err() {
                    return;
                }

                let data = buffer_clone.slice(..).get_mapped_range();

                let bytes = utils::unpad_image(&data, size.0, size.1, bpp, bytes_per_row);

                let result = ReadbackResult { bytes, size };

                tx.send(result).ok();

                drop(data);

                buffer_clone.unmap();
            });

        ReadbackHandle { receiver: rx }
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
