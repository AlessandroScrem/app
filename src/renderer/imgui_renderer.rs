use crate::asset_path;
use crate::assets::TextureId;
use crate::gpu::*;
use crate::prelude::*;
use crate::ui::{UiTexture, UiTextureResolver};
use imgui_wgpu::*;
use std::collections::HashMap;
use wgpu::*;

// registro imgui separato
pub struct ImGuiTextureRegistry {
    pub ids: HashMap<TextureId, imgui::TextureId>,
    pub framebuffer_ids: HashMap<FramebufferKind, imgui::TextureId>,
    pub shadowmap_id: Option<imgui::TextureId>,
}

impl ImGuiTextureRegistry {
    pub fn new() -> Self {
        Self {
            ids: HashMap::new(),
            framebuffer_ids: HashMap::new(),
            shadowmap_id: None,
        }
    }
}

impl UiTextureResolver for ImguiRender {
    fn resolve(&self, tex: UiTexture) -> Option<imgui::TextureId> {
        match tex {
            UiTexture::Engine(id) => self.registry.ids.get(&id).cloned(),
            UiTexture::Builtin(id) => Some(id),
            UiTexture::Framebuffer(id) => self.registry.framebuffer_ids.get(&id).cloned(),
            UiTexture::ShadowMap => self.registry.shadowmap_id,
        }
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

        // Render pass
        let mut pass = {
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ImGui Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: frame_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // non cancellare la scena
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

        // record new textures
        use imgui_wgpu::RawTextureConfig;
        for (gpu_id, tex) in texture_cache.iter() {
            if !registry.ids.contains_key(&gpu_id) {
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
                registry.ids.insert(gpu_id.clone(), id);
                debug!("add to registry texture [no name] with id {}", id.id());
            }
        }

        // rimuove quelle che non esistono più nel texture manager
        registry.ids.retain(|gpu_id, id| {
            if !texture_cache.contains_key(gpu_id) {
                renderer.textures.remove(*id);
                debug!("remove from registry [no mame] with id {}", id.id());
                false
            } else {
                true
            }
        });
    }
    pub fn sync_imgui_shadowmap<'a>(&mut self, gpu_context: &GpuContext, texture: &GpuTexture) {
        let renderer = &mut self.renderer;
        let registry = &mut self.registry.shadowmap_id;
        let device = &gpu_context.device;

        trace!("Sync_with_registry: ");
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

        if let Some(id) = registry {
            renderer.textures.replace(*id, updated_texture);
        } else {
            let id = renderer.textures.insert(updated_texture);
            *registry = Some(id.clone());
        }
    }
}
