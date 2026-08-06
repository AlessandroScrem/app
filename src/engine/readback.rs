#![allow(unused)]

use crate::gpu::gpu_readback::*;

pub enum PollResult<T> {
    Idle,
    Pending,
    Ready(T),
}

#[derive(Default)]
enum PickState {
    #[default]
    Idle,
    Pending(ReadbackHandle),
}

#[derive(Default)]
pub struct PickObject {
    state: PickState,
}

impl PickObject {
    pub fn request(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        pos: (u32, u32),
    ) {
        if !matches!(self.state, PickState::Idle) {
            return;
        }

        let handle = GpuReadback::request_readback(device, queue, texture, pos, (1, 1));

        self.state = PickState::Pending(handle);
    }
}

impl PickObject {
    pub fn poll(&mut self) -> PollResult<Option<u64>> {
        match &self.state {
            PickState::Idle => PollResult::Idle,

            PickState::Pending(handle) => match handle.try_recv() {
                None => PollResult::Pending,

                Some(result) => {
                    self.state = PickState::Idle;
                    PollResult::Ready(Self::decode(result))
                }
            },
        }
    }

    fn decode(results: ReadbackResult) -> Option<u64> {
        let id = u32::from_le_bytes([
            results.bytes[0],
            results.bytes[1],
            results.bytes[2],
            results.bytes[3],
        ]);
        if id == 0 { None } else { Some(id as u64) }
    }
}

#[derive(Default)]
enum SelectState {
    #[default]
    Idle,
    Pending(ReadbackHandle),
}

#[derive(Default)]
pub struct Select {
    state: SelectState,
}

impl Select {
    pub fn request(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        origin: (u32, u32),
        size: (u32, u32),
    ) {
        if !matches!(self.state, SelectState::Idle) {
            return;
        }

        self.state =
            SelectState::Pending(GpuReadback::request_readback(device, queue, texture, origin, size));
    }
}

impl Select {
    pub fn poll(&mut self) -> PollResult<Vec<u64>> {
        match &self.state {
            SelectState::Idle => PollResult::Idle,

            SelectState::Pending(handle) => match handle.try_recv() {
                None => PollResult::Pending,

                Some(result) => {
                    self.state = SelectState::Idle;
                    PollResult::Ready(Self::decode(result))
                }
            },
        }
    }

        // if let Some(readback) = &mut self.readback {
        //     if let Some(data) = readback.poll(&self.gpu_context.device) {
        //         println!("Buffer len: {} ", data.len());
        //         let mut ids = HashSet::new();
        //         for chunk in data.chunks_exact(8) {
        //             let id = u64::from_le_bytes(chunk.try_into().unwrap());
        //             if id != 0 {
        //                 ids.insert(EntityRawU64::from_raw_u64(id));
        //             }
        //         }
        //         bus.send_domain(Selection(SelectMulti(ids.into_iter().collect())));
        //         self.readback = None;
        //     }
        // }

    fn decode(results: ReadbackResult) -> Vec<u64> {
        results
            .bytes
            .chunks_exact(4)
            .map(|pixel| pixel[0] as u64)
            .filter(|&id| id != 0)
            .collect()
    }
}

pub enum QueryResult {
    Pick(Option<u64>),
    Selection(Vec<u64>),
}

#[derive(Default)]
pub struct ReadbackManager {
    pick: PickObject,
    selection: Select,
}

impl ReadbackManager {
    pub fn request_pick(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        pos: (u32, u32),
    ) {
        self.pick.request(device, queue, texture, pos);
    }

    pub fn request_selection(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        origin: (u32, u32),
        size: (u32, u32),
    ) {
        self.selection
            .request(device, queue, texture, origin, size);
    }

    pub fn poll_results(&mut self) -> Option<QueryResult> {
        if let PollResult::Ready(id) = self.pick.poll() {
            return Some(QueryResult::Pick(id));
        }

        if let PollResult::Ready(ids) = self.selection.poll() {
            return Some(QueryResult::Selection(ids));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use log::error;

    use super::*;
    use crate::gpu::caches::GpuTextureBuilder;
    use crate::gpu::static_textures;
    use crate::test_utils::get_device_and_queue;

    #[test]
    fn should_read_pick() {
        let (device, queue) = get_device_and_queue();
        let gpu = GpuReadback::default();

        let gpu_texture =
            GpuTextureBuilder::from_static(&static_textures::LIGHTBULB_STATIC_TEXTURE)
                .build(device, Some(queue));

        let texture = gpu_texture.inner;

        let mut pick = PickObject::default();

        pick.request(&device, &queue, &texture, (15, 15));

        let result = loop {
            GpuReadback::poll(&device);

            match pick.poll() {
                PollResult::Pending => {
                    std::thread::yield_now();
                }
                PollResult::Ready(result) => break result,
                PollResult::Idle => {}
            }
        };
        assert!(result.is_some());
    }

    #[test]
    fn should_read_pick_empty_request() {
        let mut pick = PickObject::default();

        match pick.poll() {
            PollResult::Idle => {}
            _ => panic!("expected idle"),
        }
    }

    #[test]
    fn should_ignore_multiple_pick_requests() {
        let (device, queue) = get_device_and_queue();

        let gpu = GpuReadback::default();

        let gpu_texture =
            GpuTextureBuilder::from_static(&static_textures::LIGHTBULB_STATIC_TEXTURE)
                .build(device, Some(queue));

        let texture = gpu_texture.inner;

        let mut pick = PickObject::default();

        // invio 100 richieste
        for _ in 0..100 {
            pick.request(&device, &queue, &texture, (15, 15));
        }

        let mut count = 0;

        loop {
            GpuReadback::poll(&device);

            match pick.poll() {
                PollResult::Pending => {
                    std::thread::yield_now();
                }
                PollResult::Ready(result) => {
                    count += 1;
                }
                PollResult::Idle => {
                    break;
                }
            }
        }

        assert_eq!(count, 1);
    }

    #[test]
    fn should_read_select() {
        let (device, queue) = get_device_and_queue();
        let gpu = GpuReadback::default();

        let gpu_texture =
            GpuTextureBuilder::from_static(&static_textures::LIGHTBULB_STATIC_TEXTURE)
                .build(device, Some(queue));

        let texture = gpu_texture.inner;

        let mut select = Select::default();

        select.request(&device, &queue, &texture, (0, 0), (10, 8));

        let result = loop {
            GpuReadback::poll(&device);

            match select.poll() {
                PollResult::Pending => {
                    std::thread::yield_now();
                }
                PollResult::Ready(result) => break result,
                PollResult::Idle => {}
            }
        };

        assert_eq!(result.len(), 7);
    }

    #[test]
    fn should_request_manager_results() {
        let (device, queue) = get_device_and_queue();
        let gpu = GpuReadback::default();

        let gpu_texture =
            GpuTextureBuilder::from_static(&static_textures::LIGHTBULB_STATIC_TEXTURE)
                .build(device, Some(queue));

        let texture = gpu_texture.inner;

        let mut req_mgr = ReadbackManager::default();

        req_mgr.request_selection(&device, &queue, &texture, (0, 0), (10, 8));

        req_mgr.request_pick(&device, &queue, &texture, (15, 15));

        let mut pick_received = false;
        let mut selection_received = false;

        loop {
            GpuReadback::poll(&device);

            if let Some(result) = req_mgr.poll_results() {
                match result {
                    QueryResult::Pick(id) => {
                        pick_received = true;
                    }

                    QueryResult::Selection(ids) => {
                        selection_received = true;
                    }
                }
            }

            if pick_received && selection_received {
                break;
            }

            std::thread::yield_now();
        }

        assert!(pick_received);
        assert!(selection_received);
    }
}
