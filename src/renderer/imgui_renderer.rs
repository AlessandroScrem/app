use super::*;

use imgui_wgpu::*;
use std::collections::HashMap;
use wgpu::*;

pub(crate) enum UiTexture {
    Engine(TextureId),         // la texture viene dall’engine
    Builtin(imgui::TextureId), // icone, font, ecc.
}

pub(crate) trait UiTextureResolver {
    fn resolve(&self, tex: UiTexture) -> Option<imgui::TextureId>;
}

// registro imgui separato
pub(crate) struct ImGuiTextureRegistry {
    pub(crate) ids: HashMap<TextureId, imgui::TextureId>,
}

impl ImGuiTextureRegistry {
    pub(crate) fn new() -> Self {
        Self {
            ids: HashMap::new(),
        }
    }
}
pub(crate) struct ImguiRender {
    pub(crate) renderer: imgui_wgpu::Renderer,
    pub(crate) registry: ImGuiTextureRegistry,
}

impl ImguiRender {
    pub(crate) fn new(
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

        context.fonts().add_font(&[imgui::FontSource::DefaultFontData {
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
    pub(crate) fn render(
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
