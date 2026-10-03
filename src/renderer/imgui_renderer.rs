use crate::asset_path;
use crate::ResourceId;
use crate::gpu::*;
use crate::prelude::*;
use crate::ui::UiTextureRegistry;
use imgui_wgpu::*;
use std::collections::{HashMap, HashSet};
use wgpu::*;

pub struct ImGuiTextureRegistry {
    textures: HashMap<ResourceId, imgui::TextureId>,
    asset_ids: HashSet<ResourceId>,
    shadowmap_id: ResourceId,
    material_preview_id: ResourceId,
}

impl ImGuiTextureRegistry {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            asset_ids: HashSet::new(),
            shadowmap_id: ResourceId::new(),
            material_preview_id: ResourceId::new(),
        }
    }

    pub fn ui_registry(&self) -> UiTextureRegistry {
        let mut registry = UiTextureRegistry::default();
        for (&resource, &id) in &self.textures {
            registry.set_texture(resource, id);
        }
        registry.set_shadow_map(self.textures.get(&self.shadowmap_id).copied());
        registry.set_material_preview(self.textures.get(&self.material_preview_id).copied());
        registry
    }
}

pub struct ImguiRender {
    pub renderer: imgui_wgpu::Renderer,
    pub registry: ImGuiTextureRegistry,
}

impl ImguiRender {
    pub fn new(
        device: &Device,
        queue: &Queue,
        window: &winit::window::Window,
        context: &mut imgui::Context,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        let renderer_config = RendererConfig {
            texture_format,
            ..Default::default()
        };

        let hidpi_factor = window.scale_factor();
        println!("Scale factor {}", hidpi_factor);
        let font_size = (9.0 * hidpi_factor) as f32;

        context.fonts().add_font(&[
            imgui::FontSource::DefaultFontData {
                config: Some(imgui::FontConfig {
                    name: Some("Default".into()),
                    oversample_h: 1,
                    pixel_snap_h: true,
                    size_pixels: font_size,
                    ..Default::default()
                }),
            },
            imgui::FontSource::TtfData {
                data: include_bytes!(asset_path!("fonts/codicon.ttf")),
                size_pixels: 9.0,
                config: Some(imgui::FontConfig {
                    name: Some("Codicons".into()),
                    pixel_snap_h: true,
                    oversample_h: 1,
                    glyph_ranges: imgui::FontGlyphRanges::from_slice(&[0xEA60, 0xEC1E, 0]),
                    ..imgui::FontConfig::default()
                }),
            },
        ]);

        let renderer = imgui_wgpu::Renderer::new(context, &device, &queue, renderer_config);
        let registry = ImGuiTextureRegistry::new();

        Self { renderer, registry }
    }

    pub fn render(
        &mut self,
        draw_data: &imgui::DrawData,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        device: &Device,
        queue: &Queue,
    ) {
        let frame_view = target;

        let mut pass = {
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ImGui Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: frame_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            })
        };

        match self.renderer.render(draw_data, queue, device, &mut pass) {
            Ok(()) => {}
            Err(e) => {
                error!("Imgui Render failed: {:?}", e);
            }
        }
    }
}

impl ImguiRender {
    pub fn sync_imgui_texture(
        &mut self,
        gpu_context: &GpuContext,
        texture_cache: &GpuTextureCache,
    ) {
        let renderer = &mut self.renderer;
        let registry = &mut self.registry;
        let device = &gpu_context.device;

        debug!("Sync_with_registry: ");

        use imgui_wgpu::RawTextureConfig;
        for (gpu_id, tex) in texture_cache.iter() {
            if !registry.textures.contains_key(gpu_id) {
                let texture_config = RawTextureConfig {
                    label: None,
                    sampler_desc: wgpu::SamplerDescriptor {
                        mag_filter: wgpu::FilterMode::Linear,
                        min_filter: wgpu::FilterMode::Linear,
                        mipmap_filter: wgpu::MipmapFilterMode::Linear,
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
                registry.textures.insert(*gpu_id, id);
                registry.asset_ids.insert(*gpu_id);
                debug!("add to registry texture with id {}", id.id());
            }
        }

        registry.asset_ids.retain(|gpu_id| {
            if texture_cache.contains_key(gpu_id) {
                true
            } else {
                if let Some(id) = registry.textures.remove(gpu_id) {
                    renderer.textures.remove(id);
                    debug!("remove from registry texture with id {}", id.id());
                }
                false
            }
        });
    }

    pub fn sync_imgui_material_preview(
        &mut self,
        gpu_context: &GpuContext,
        texture: std::sync::Arc<wgpu::Texture>,
        view: std::sync::Arc<wgpu::TextureView>,
        extent: wgpu::Extent3d,
    ) {
        let renderer = &mut self.renderer;
        let resource_id = self.registry.material_preview_id;
        let registry = &mut self.registry.textures;
        let device = &gpu_context.device;

        let texture_config = RawTextureConfig {
            label: Some("Material Preview"),
            sampler_desc: wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::MipmapFilterMode::Linear,
                ..Default::default()
            },
        };

        let updated_texture = imgui_wgpu::Texture::from_raw_parts(
            device,
            renderer,
            texture,
            view,
            None,
            Some(&texture_config),
            extent,
        );

        let id = registry
            .get(&resource_id)
            .copied()
            .unwrap_or_else(|| renderer.textures.insert(updated_texture));
        if registry.insert(resource_id, id).is_some() {
            renderer.textures.replace(id, updated_texture);
        }
    }

    pub fn sync_imgui_shadowmap(&mut self, gpu_context: &GpuContext, texture: &GpuTexture) {
        let renderer = &mut self.renderer;
        let resource_id = self.registry.shadowmap_id;
        let registry = &mut self.registry.textures;
        let device = &gpu_context.device;

        let texture_config = RawTextureConfig {
            label: None,
            sampler_desc: wgpu::SamplerDescriptor {
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::MipmapFilterMode::Nearest,
                ..Default::default()
            },
        };
        let updated_texture = imgui_wgpu::Texture::from_raw_parts(
            device,
            renderer,
            texture.inner.clone(),
            texture.view.clone(),
            None,
            Some(&texture_config),
            texture.extent,
        );

        let id = registry
            .get(&resource_id)
            .copied()
            .unwrap_or_else(|| renderer.textures.insert(updated_texture));
        if registry.insert(resource_id, id).is_some() {
            renderer.textures.replace(id, updated_texture);
        }
    }
}
