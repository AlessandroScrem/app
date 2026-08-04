use std::{collections::HashSet, sync::mpsc};

#[derive(Clone, Copy)]
pub enum ReadbackState {
    Idle,          // nessuna copia in corso
    CopySubmitted, // la GPU sta scrivendo nel buffer
    Mapping,       // map_async richiesto, attendo callback
    Ready(Option<u64>),
}

pub type PickingCoords = (u32, u32);
pub struct PickObject {
    pub buffer: wgpu::Buffer,
    pub picking_coords: PickingCoords,
    pub state: ReadbackState,
    readback_tx: mpsc::Sender<()>,
    readback_rx: mpsc::Receiver<()>,
    pub picking_size: Option<(u32, u32)>,
}

pub const BYTE_PER_PIXEL: u32 = std::mem::size_of::<u64>() as u32;

pub fn align_bytes_per_row(value: u32) -> u32 {
    const ALIGNEMENT: u32 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    ((value + ALIGNEMENT - 1) / ALIGNEMENT) * ALIGNEMENT
}

impl PickObject {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let unaligned_row = width * BYTE_PER_PIXEL;
        let size = (align_bytes_per_row(unaligned_row) * height) as u64;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("buffer Readback Pixels"),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let (tx, rx) = mpsc::channel();
        Self {
            buffer,
            state: ReadbackState::Idle,
            readback_tx: tx,
            readback_rx: rx,
            picking_coords: (0, 0),
            picking_size: None,
        }
    }

    pub fn set_picking_coords(&mut self, coords: PickingCoords) {
        self.picking_coords = coords;
    }

    pub fn get_picking_coords(&self) -> PickingCoords {
        self.picking_coords
    }

    pub fn poll_readback(&mut self, device: &wgpu::Device) -> ReadbackState {
        let _ = device.poll(wgpu::PollType::Poll);

        match self.state {
            ReadbackState::Idle => { self.state}

            ReadbackState::CopySubmitted => {
                let slice = self.buffer.slice(..);

                let tx = self.readback_tx.clone();

                slice.map_async(wgpu::MapMode::Read, move |res| {
                    if res.is_ok() {
                        let _ = tx.send(());
                    }
                });

                self.state = ReadbackState::Mapping;
                self.state
            }

            ReadbackState::Mapping => {
                if self.readback_rx.try_recv().is_ok() {
                    let id = {
                        let slice = self.buffer.slice(..);
                        let data = slice.get_mapped_range();

                        // Self::read_pixel(data)
                        Self::read_pixels(&data, self.picking_size.unwrap_or((1, 1))).iter().nth(0).copied()
                    };

                    self.buffer.unmap();
                    self.state = ReadbackState::Ready(id);
                }
                self.state
            }
            ReadbackState::Ready(id) => {
                let result = ReadbackState::Ready(id); 
                self.state = ReadbackState::Idle;
                result
            }
        }
    }

    // read single pixel
    fn read_pixel(data: wgpu::BufferView) -> Option<u64> {
        let size = BYTE_PER_PIXEL as usize;
        if data.len() >= size {
            let id = u64::from_le_bytes(
                data[0..size]
                    .try_into()
                    .expect("unable to convert pixel data"),
            );
            Some(id)
        } else {
            None
        }
    }

    // read rect
    fn read_pixels(data: &[u8], size: (u32, u32)) -> HashSet<u64> {
        let width = size.0 as usize;
        let height = size.0 as usize;
        let pixel_size = BYTE_PER_PIXEL as usize;

        let row_size = align_bytes_per_row((width * pixel_size) as u32) as usize;
        let required_size = row_size * height;
        let mut result = HashSet::new();

        if data.len() >= required_size as usize {
            for row in data.chunks(row_size).take(height as usize) {
                for pixel in row.chunks_exact(pixel_size) {
                    let id =
                        u64::from_le_bytes(pixel.try_into().expect("unable to convert pixel data"));
                    if id != 0 {
                        result.insert(id);
                    }
                }
            }
        }
        result
    }
}
