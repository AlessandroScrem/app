use std::time::Instant;

use crate::{assets::texture_manager::TextureManager, prelude::imgui_tools::ImGuiTextureRegistry};
use legion::*;
use log::debug;

#[system]
pub fn registry_update(
    #[resource] device: &wgpu::Device,
    #[resource] manager: &TextureManager,
    #[resource] registry: &mut ImGuiTextureRegistry,
    #[resource] renderer: &mut imgui_wgpu::Renderer,
) {
    let timer = Instant::now();

    sync_with_registry(&device, &manager, registry, renderer);

    debug!("Time for sync_with_registry: {:?}", timer.elapsed());
}

// Sync texture with TextureManager textures
// TODO: maybe use an event handler for avoid to sync each frame
// bub sync when add or removing textures from texture_manager
pub fn sync_with_registry(
    device: &wgpu::Device,
    manager: &TextureManager,
    registry: &mut ImGuiTextureRegistry,
    renderer: &mut imgui_wgpu::Renderer,
) {
    // record new textures
    use imgui_wgpu::RawTextureConfig;
    for (path, tex) in &manager.textures {
        if !registry.ids.contains_key(path) {
            let texture_config = RawTextureConfig {
                label: None,
                sampler_desc: wgpu::SamplerDescriptor {
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Linear,
                    mipmap_filter: wgpu::FilterMode::Linear,
                    ..Default::default()
                },
            };
            let id = renderer
                .textures
                .insert(imgui_wgpu::Texture::from_raw_parts(
                    device,
                    renderer,
                    tex.inner.clone(),
                    tex.view.clone(),
                    None,
                    Some(&texture_config),
                    tex.extent,
                ));
            registry.ids.insert(path.clone(), id);
            debug!("add to registry {} with id {}", path.display(), id.id());
        }
    }

    // rimuove quelle che non esistono più nel texture manager
    registry.ids.retain(|path, id| {
        if !manager.textures.contains_key(path) {
            renderer.textures.remove(*id);
            debug!(
                "remove from registry {} with id {}",
                path.display(),
                id.id()
            );
            false
        } else {
            true
        }
    });
}
