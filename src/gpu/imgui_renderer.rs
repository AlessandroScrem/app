use super::*;

use imgui_wgpu::*;
use std::collections::HashMap;
use wgpu::*;
use crate::assets::TextureId;

#[allow(dead_code)]
pub enum UiTexture {
    Engine(TextureId),         // la texture viene dall’engine
    Builtin(imgui::TextureId), // icone, font, ecc.
}

pub trait UiTextureResolver {
    fn resolve(&self, tex: UiTexture) -> Option<imgui::TextureId>;
}

// registro imgui separato
pub struct ImGuiTextureRegistry {
    pub ids: HashMap<TextureId, imgui::TextureId>,
}

impl ImGuiTextureRegistry {
    pub fn new() -> Self {
        Self {
            ids: HashMap::new(),
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
        let font_size = (9.0 * hidpi_factor) as f32;

        context
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData {
                config: Some(imgui::FontConfig {
                    oversample_h: 1,
                    pixel_snap_h: true,
                    size_pixels: font_size,
                    ..Default::default()
                }),
            }]);

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

impl UiTextureResolver for ImguiRender {
    fn resolve(&self, tex: UiTexture) -> Option<imgui::TextureId> {
        match tex {
            UiTexture::Engine(id) => self.registry.ids.get(&id).cloned(),
            UiTexture::Builtin(id) => Some(id),
        }
    }
}
