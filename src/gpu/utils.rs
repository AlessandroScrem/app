pub fn copy_texture_to_cpu(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
) -> anyhow::Result<Vec<u8>> {
    // copy full texture to buffer.
    let width = texture.width();
    let height = texture.height();
    let format = texture.format();
    let byte_per_pixel = bytes_per_pixel(format)?;

    // Bytes per row per WGPU (padded a 256)
    let bytes_per_row = width * byte_per_pixel;
    let padded_bytes_per_row = align_to(bytes_per_row, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u32);

    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        size: (padded_bytes_per_row * width) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        label: None,
        mapped_at_creation: false,
    });

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            aspect: wgpu::TextureAspect::All,
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &output_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            depth_or_array_layers: 1,
            width,
            height,
        },
    );

    queue.submit(Some(encoder.finish()));

    let slice = output_buffer.slice(..);

    // The mapping process is async, so we'll need to create a channel to get
    // the success flag for our mapping
    let (tx, rx) = std::sync::mpsc::channel();

    // We send the success or failure of our mapping via a callback
    slice.map_async(wgpu::MapMode::Read, move |result| {
        tx.send(result).unwrap();
    });

    // The callback we submitted to map async will only get called after the
    // device is polled or the queue submitted
    device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    })?;

    // We check if the mapping was successful here
    rx.recv()??;

    let mapped = slice.get_mapped_range();

    let raw = unpad_image(&mapped, width, height, byte_per_pixel, padded_bytes_per_row);
    drop(mapped);
    output_buffer.unmap();

    Ok(match format {
        wgpu::TextureFormat::Rgba8Unorm
        | wgpu::TextureFormat::Rgba8UnormSrgb
        | wgpu::TextureFormat::Rg32Uint => raw,

        wgpu::TextureFormat::Rg16Float => rg16float_to_rgba8(&raw, width, height),

        wgpu::TextureFormat::Rgba16Float => rgba16float_to_rgba8(&raw, width, height),

        _ => unreachable!(),
    })
}

pub fn bytes_per_pixel(format: wgpu::TextureFormat) -> anyhow::Result<u32> {
    Ok(match format {
        wgpu::TextureFormat::Rgba8Unorm
        | wgpu::TextureFormat::Rgba8UnormSrgb
        | wgpu::TextureFormat::Rg16Float => 4,

        wgpu::TextureFormat::Rg32Uint | wgpu::TextureFormat::Rgba16Float => 8,

        _ => anyhow::bail!("Unsupported texture format {format:?}"),
    })
}

/// Align `value` up to the nearest multiple of `alignment`.
/// `alignment` must be a power of two.
///
pub fn align_to(value: u32, alignment: u32) -> u32 {
    ((value + alignment - 1) / alignment) * alignment
}

/// Remove padding from data read from texture  eg: copy_texture_to_buffer().
/// `data` slice mapped from GPU buffer.
/// `width` = in pixel
/// `height` = in pixel
/// `bytes_per_pixel` = how many byte per pixel (es. RGBA8 = 4, RGBA16F = 8, ecc.)
/// `bytes_per_row_padded` = row pitch returned from wgpu
///
pub fn unpad_image(
    data: &[u8],
    width: u32,
    height: u32,
    bytes_per_pixel: u32,
    bytes_per_row_padded: u32,
) -> Vec<u8> {
    let bytes_per_row_unpadded = width * bytes_per_pixel;
    let mut unpadded = vec![0u8; (bytes_per_row_unpadded * height) as usize];

    for y in 0..height as usize {
        let src_start = y * bytes_per_row_padded as usize;
        let dst_start = y * bytes_per_row_unpadded as usize;

        unpadded[dst_start..dst_start + bytes_per_row_unpadded as usize]
            .copy_from_slice(&data[src_start..src_start + bytes_per_row_unpadded as usize]);
    }

    unpadded
}

/// Convert RG16F data to RGBA8 data.
/// `raw` = input data slice
/// `width` = in pixel
/// `height` = in pixel
/// Returns a Vec<u8> with RGBA8 data.
/// The B channel is set to 0, and A channel to 255.
/// Each pixel in input is 4 bytes (2 half-floats), output is 4 bytes (4 u8).
/// Clamps values to [0,1] before conversion.
/// # Panics
/// Panics if `raw` length is not equal to width * height * 4.
///
fn rg16float_to_rgba8(raw: &[u8], width: u32, height: u32) -> Vec<u8> {
    use half::f16;
    let mut out = Vec::with_capacity((width * height * 4) as usize);

    for i in 0..(width * height) as usize {
        // Ogni pixel = 4 byte = 2 half-float
        let offset = i * 4;
        let r_half = u16::from_le_bytes([raw[offset], raw[offset + 1]]);
        let g_half = u16::from_le_bytes([raw[offset + 2], raw[offset + 3]]);

        let r = f16::from_bits(r_half).to_f32();
        let g = f16::from_bits(g_half).to_f32();

        // Converti in [0,255]
        let r_u8 = (r.clamp(0.0, 1.0) * 255.0).round() as u8;
        let g_u8 = (g.clamp(0.0, 1.0) * 255.0).round() as u8;

        // B=0, A=255 (o come preferisci)
        out.push(r_u8);
        out.push(g_u8);
        out.push(0);
        out.push(255);
    }

    out
}
/// Convert RGBA16F data to RGBA8 data.
/// `raw` = input data slice
/// `width` = in pixel
/// `height` = in pixel
/// Returns a Vec<u8> with RGBA8 data.
/// Each pixel in input is 8 bytes (4 half-floats), output is 4 bytes (4 u8).
/// Clamps values to [0,1] before conversion.
/// # Panics
/// Panics if `raw` length is not equal to width * height * 8.
///
fn rgba16float_to_rgba8(raw: &[u8], width: u32, height: u32) -> Vec<u8> {
    use half::f16;
    // Ogni pixel = 8 byte = 4 half-float
    let pixel_size = 8;
    let mut out = Vec::with_capacity((width * height * pixel_size) as usize);

    for i in 0..(width * height) as usize {
        let offset = i * pixel_size as usize;
        let r_half = u16::from_le_bytes([raw[offset], raw[offset + 1]]);
        let g_half = u16::from_le_bytes([raw[offset + 2], raw[offset + 3]]);
        let b_half = u16::from_le_bytes([raw[offset + 4], raw[offset + 5]]);
        let a_half = u16::from_le_bytes([raw[offset + 6], raw[offset + 7]]);

        let r = f16::from_bits(r_half).to_f32();
        let g = f16::from_bits(g_half).to_f32();
        let b = f16::from_bits(b_half).to_f32();
        let a = f16::from_bits(a_half).to_f32();

        // Converti in [0,255]
        let r_u8 = (r.clamp(0.0, 1.0) * 255.0).round() as u8;
        let g_u8 = (g.clamp(0.0, 1.0) * 255.0).round() as u8;
        let b_u8 = (b.clamp(0.0, 1.0) * 255.0).round() as u8;
        let a_u8 = (a.clamp(0.0, 1.0) * 255.0).round() as u8;

        // B=0, A=255 (o come preferisci)
        out.push(r_u8);
        out.push(g_u8);
        out.push(b_u8);
        out.push(a_u8);
    }

    out
}

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub enum ReadbackState {
    Idle,
    Waiting(wgpu::Buffer),
    Ready(Vec<u8>),
}

pub struct TextureReadback {
    state: ReadbackState,
    ready: Arc<AtomicBool>,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
}

impl TextureReadback {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        position: (u32, u32),
        width: u32,
        height: u32,
    ) -> Self {
        let width = width.min(texture.width() - position.0);
        let height = height.min(texture.height() - position.1);

        let ready = Arc::new(AtomicBool::new(false));
        let state = Self::request(
            device,
            queue,
            texture,
            ready.clone(),
            position,
            width,
            height,
        );

        Self {
            state,
            ready,
            format: texture.format(),
            width,
            height,
        }
    }

    fn request(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        ready_flag: Arc<AtomicBool>,
        position: (u32, u32),
        width: u32,
        height: u32,
    ) -> ReadbackState {
        let bpp = bytes_per_pixel(texture.format()).unwrap();

        let bytes_per_row = align_to(width * bpp, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("texture_readback"),
            size: (bytes_per_row * height) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("texture_readback_encoder"),
        });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: position.0,
                    y: position.1,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        queue.submit(Some(encoder.finish()));

        buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                if result.is_ok() {
                    ready_flag.store(true, Ordering::Release);
                }
            });

        ReadbackState::Waiting(buffer)
    }

    pub fn poll(&mut self, device: &wgpu::Device) -> Option<Vec<u8>> {
        device.poll(wgpu::PollType::Poll).ok();

        match &mut self.state {
            ReadbackState::Idle => None,

            ReadbackState::Waiting(buffer) => {
                if !self.ready.load(Ordering::Acquire) {
                    return None;
                }

                let bpp = bytes_per_pixel(self.format).unwrap();

                let row = align_to((self.width) * bpp, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);

                let mapped = buffer.slice(..).get_mapped_range();

                let raw = unpad_image(&mapped, self.width, self.height, bpp, row);

                drop(mapped);

                buffer.unmap();
                self.ready.store(false, Ordering::Release);

                let result = match self.format {
                    wgpu::TextureFormat::Rg16Float => {
                        rg16float_to_rgba8(&raw, self.width, self.height)
                    }

                    wgpu::TextureFormat::Rgba16Float => {
                        rgba16float_to_rgba8(&raw, self.width, self.height)
                    }

                    _ => raw,
                };

                self.state = ReadbackState::Ready(result);

                None
            }

            ReadbackState::Ready(_) => {
                let old = std::mem::replace(&mut self.state, ReadbackState::Idle);

                match old {
                    ReadbackState::Ready(data) => Some(data),

                    _ => None,
                }
            }
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu::caches::GpuTextureBuilder;
    use crate::gpu::static_textures;
    use crate::test_utils::get_device_and_queue;

    fn wait_readback(readback: &mut TextureReadback, device: &wgpu::Device) -> Vec<u8> {
        use std::time::Instant;

        let start = Instant::now();
        let mut cycles = 0;
        loop {
            cycles += 1;
            if let Some(data) = readback.poll(device) {
                println!(
                    "Readback pronto dopo {} cicli in {:?}",
                    cycles,
                    start.elapsed()
                );
                return data;
            }
            std::thread::yield_now();
        }
    }

    #[test]
    fn should_readback() {
        let (device, queue) = get_device_and_queue();

        let texture = GpuTextureBuilder::from_static(&static_textures::WHITE_STATIC_TEXTURE)
            .build(device, Some(queue))
            .inner;
        let width = texture.width();
        let height = texture.height();

        let mut readback = TextureReadback::new(device, queue, &texture, (0, 0), width, height);

        let result = wait_readback(&mut readback, device);

        println!("Read {} bytes", result.len());

        assert!(!result.is_empty());
    }
}
