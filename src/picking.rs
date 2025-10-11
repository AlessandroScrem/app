use std::sync::{Arc, atomic::AtomicU64};

use legion::Entity;

use crate::entities::EntityRawU64;

pub struct PickObject {
    pub selected: Option<Entity>,
    pub hovered: Option<Entity>,
    pub buffer: PickBuffer,
}

impl PickObject {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            selected: None,
            hovered: None,
            buffer: PickBuffer::new(device),
        }
    }

    pub fn apply(&mut self) {
        self.buffer.read_id();
        if let Some(id) = self.buffer.get_id_if_ready() {
            self.hovered = Some(Entity::from_raw_u64(id));
            // println!("Hovered is {:?}", self.hovered);
        }
    }

    pub fn select_hovered(&mut self) {
        self.selected = self.hovered;
        // println!("Selected is {:?}", self.selected);
    }
    pub fn select(&mut self, select: Option<Entity>) {
        self.selected = select;
        // println!("Selected is {:?}", self.selected);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickState {
    Idle,    // pronto per una nuova copia
    Copying, // in corso: la GPU sta scrivendo nel buffer
    Mapped,  // mappato e pronto da leggere
}

pub struct PickBuffer {
    pub buffer: Arc<wgpu::Buffer>,
    pub last_id: Arc<AtomicU64>,
    pub state: Arc<std::sync::Mutex<PickState>>,
}

impl PickBuffer {
    pub fn new(device: &wgpu::Device) -> Self {
        let buffer = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("buffer Readback Pixel"),
            size: 256,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        }));
        let last_id = Arc::new(AtomicU64::new(0));
        Self {
            buffer,
            last_id,
            state: Arc::new(std::sync::Mutex::new(PickState::Idle)),
        }
    }

    pub fn ready(&self) -> bool {
        let state = self.state.lock().unwrap();
        *state == PickState::Idle
    }

    fn read_id(&self) {
        use std::sync::atomic::Ordering;

        let mut state = self.state.lock().unwrap();
        if *state != PickState::Idle {
            // Evita doppio map se non ancora completato
            return;
        }

        *state = PickState::Copying;
        // println!("State is {:?}", *state);

        let buffer = Arc::clone(&self.buffer);
        let last_id = Arc::clone(&self.last_id);
        let buffer_clone = Arc::clone(&buffer);
        let state_arc = Arc::clone(&self.state);

        // Map_async direttamente sul buffer
        buffer_clone
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |res| {
                let mut state = state_arc.lock().unwrap();
                if let Ok(()) = res {
                    let data = buffer.slice(..).get_mapped_range();
                    if data.len() >= 8 {
                        let id = u64::from_le_bytes(data[0..8].try_into().unwrap());
                        last_id.store(id, Ordering::Relaxed);
                    }

                    drop(data);
                    buffer.unmap();
                    *state = PickState::Mapped;
                    // println!("State is {:?}", *state);
                }
            });
    }

    /// Ritorna l'ultimo ID valido (se pronto)
    fn get_id_if_ready(&self) -> Option<u64> {
        use std::sync::atomic::Ordering;

        let mut state = self.state.lock().unwrap();
        if *state == PickState::Mapped {
            *state = PickState::Idle;
            // println!("State is {:?}", *state);
            Some(self.last_id.load(Ordering::Relaxed))
        } else {
            None
        }
    }
}
