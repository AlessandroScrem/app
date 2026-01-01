use std::time::Instant;

use crate::{assets::texture_manager::TextureManager, prelude::ui::ImguiState};
use legion::*;
use log::debug;

#[system]
pub fn registry_update(
    #[resource] device: &wgpu::Device,
    #[resource] manager: &TextureManager,
    #[resource] imgui: &mut ImguiState,
) {
    let timer = Instant::now();

    imgui.sync_with_registry(&device, &manager);

    debug!("Time for sync_with_registry: {:?}", timer.elapsed());
}
