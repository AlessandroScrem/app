use std::sync::mpsc;

pub enum ReadbackState {
    Idle,          // nessuna copia in corso
    CopySubmitted, // la GPU sta scrivendo nel buffer
    Mapping,       // map_async richiesto, attendo callback
}

pub type PickingCoords = (u32, u32);
pub struct PickObject {
    pub buffer: wgpu::Buffer,
    pub picking_coords: PickingCoords,
    pub state: ReadbackState,
    cached_id: Option<u64>,
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
            buffer,
            state: ReadbackState::Idle,
            cached_id: None,
            readback_tx: tx,
            readback_rx: rx,
            picking_coords: (0,0),
        }
    }

    pub fn set_picking_coords(&mut self, coords: PickingCoords) {
        self.picking_coords = coords;
    }

    pub fn get_picking_coords(&self) -> PickingCoords {
        self.picking_coords
    }

    pub fn poll_readback(&mut self, device: &wgpu::Device) -> Option<u64> {
        let _ = device.poll(wgpu::PollType::Poll);

        match self.state {
            ReadbackState::Idle => {}

            ReadbackState::CopySubmitted => {
                let slice = self.buffer.slice(..);

                let tx = self.readback_tx.clone();

                slice.map_async(wgpu::MapMode::Read, move |res| {
                    if res.is_ok() {
                        let _ = tx.send(());
                    }
                });

                self.state = ReadbackState::Mapping;
            }

            ReadbackState::Mapping => {
                if self.readback_rx.try_recv().is_ok() {
                    let id = {
                        let slice = self.buffer.slice(..);
                        let data = slice.get_mapped_range();

                        Self::read_pixel(data)
                    };

                    self.buffer.unmap();

                    self.cached_id = id;

                    self.state = ReadbackState::Idle;
                }
            }
        }

        self.cached_id
    }

    fn read_pixel(data: wgpu::BufferView) -> Option<u64> {
        if data.len() >= 8 {
            let id =
                u64::from_le_bytes(data[0..8].try_into().expect("unable to convert pixel data"));
            Some(id)
        } else {
            None
        }
    }
}
