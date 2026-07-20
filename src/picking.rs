use legion::Entity;
use wgpu::Device;

use std::sync::mpsc;

pub type PickingData = (u32, u32);
pub struct PickObject {
    pub buffer: wgpu::Buffer,
    pub picking_coords: Option<PickingData>,
    pending: bool,
    readback_tx: mpsc::Sender<()>,
    readback_rx: mpsc::Receiver<()>,
}

impl PickObject {
    pub fn new(device: &wgpu::Device) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("buffer Readback Pixel"),
            size: 256,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let (tx, rx) = mpsc::channel();
        Self {
            pending: false,
            buffer,
            readback_tx: tx,
            readback_rx: rx,
            picking_coords: None,
        }
    }

    pub fn is_ready(&self) -> bool {
        !self.pending
    }

    pub fn set_picking_coords(&mut self, coords: PickingData) {
        self.picking_coords = self.is_ready().then_some(coords);
    }

    pub fn get_picking_coords(&self) -> Option<PickingData> {
        self.picking_coords
    }

    pub fn poll_readback(&mut self, device: &Device) -> Option<Entity> {
        self.request_readback();

        let mut entity: Option<Entity> = None;
        // Avanza stato GPU
        let _ = device.poll(wgpu::PollType::Poll);

        if self.pending && self.readback_rx.try_recv().is_ok() {
            {
                let slice = self.buffer.slice(..);
                let data = slice.get_mapped_range(); //CPU READ
                entity = Self::read_pixel(data);
                // println!("Read {:?} ", entity);
            }

            self.buffer.unmap();
            self.pending = false;
        }

        entity
    }

    fn read_pixel(data: wgpu::BufferView) -> Option<Entity> {
        if data.len() >= 8 {
            let id =
                u64::from_le_bytes(data[0..8].try_into().expect("unable to convert pixel data"));
            let entity: Entity = crate::EntityRawU64::from_raw_u64(id);
            Some(entity)
        } else {
            None
        }
    }

    fn request_readback(&mut self) {
        if self.is_ready() {
            let slice = self.buffer.slice(..);
            let tx = self.readback_tx.clone();

            slice.map_async(wgpu::MapMode::Read, move |res| {
                if res.is_ok() {
                    let _ = tx.send(());
                }
            });

            self.pending = true;
        }
    }
}
