use crate::ResourceId;
use crate::asset_path;
use crate::gpu::*;
use crate::prelude::*;
use crate::ui::UiTextures;
use imgui_wgpu::*;
use std::collections::HashMap;
use wgpu::*;

pub struct ImGuiTextureRegistry {
    textures: HashMap<ResourceId, imgui::TextureId>,
}

impl ImGuiTextureRegistry {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }

    pub fn add(
        &mut self,
        renderer: &mut imgui_wgpu::Renderer,
        device: &Device,
        resource_id: ResourceId,
        texture: &GpuTexture,
    ) -> imgui::TextureId {
        let texture_config = RawTextureConfig {
            label: None,
            sampler_desc: wgpu::SamplerDescriptor {
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::MipmapFilterMode::Linear,
                ..Default::default()
            },
        };
        let imgui_texture = imgui_wgpu::Texture::from_raw_parts(
            device,
            renderer,
            texture.inner.clone(),
            texture.view.clone(),
            None,
            Some(&texture_config),
            texture.extent,
        );

        if let Some(id) = self.textures.get(&resource_id).copied() {
            renderer.textures.replace(id, imgui_texture);
            id
        } else {
            let id = renderer.textures.insert(imgui_texture);
            self.textures.insert(resource_id, id);
            id
        }
    }

    pub fn remove(
        &mut self,
        renderer: &mut imgui_wgpu::Renderer,
        resource_id: ResourceId,
    ) -> bool {
        let Some(id) = self.textures.remove(&resource_id) else {
            return false;
        };

        renderer.textures.remove(id);
        true
    }

    pub fn get(&self, resource_id: ResourceId) -> Option<imgui::TextureId> {
        self.textures.get(&resource_id).copied()
    }

    pub fn ui_textures(&self, shadow_map: ResourceId, material_preview: ResourceId) -> UiTextures {
        UiTextures::new(
            self.textures.clone(),
            self.textures.get(&shadow_map).copied(),
            self.textures.get(&material_preview).copied(),
        )
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
    pub fn sync_imgui_texture_cache(
        &mut self,
        gpu_context: &GpuContext,
        texture_cache: &mut GpuTextureCache,
    ) {
        let device = &gpu_context.device;
        let events: Vec<_> = texture_cache.drain_events().collect();

        for event in events {
            match event {
                GpuTextureEvent::Added(resource_id) => {
                    if let Some(texture) = texture_cache.get(resource_id) {
                        self.registry
                            .add(&mut self.renderer, device, resource_id, texture);
                    }
                }
                GpuTextureEvent::Removed(resource_id) => {
                    self.registry.remove(&mut self.renderer, resource_id);
                }
            }
        }
    }

    pub fn sync_imgui_texture(
        &mut self,
        gpu_context: &GpuContext,
        resource_id: ResourceId,
        texture: std::sync::Arc<wgpu::Texture>,
        view: std::sync::Arc<wgpu::TextureView>,
        extent: wgpu::Extent3d,
    ) {
        let renderer = &mut self.renderer;
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

        if let Some(id) = registry.get(&resource_id).copied() {
            renderer.textures.replace(id, updated_texture);
        } else {
            let id = renderer.textures.insert(updated_texture);
            registry.insert(resource_id, id);
        }
    }

    pub fn sync_imgui_texture_from_gpu(
        &mut self,
        gpu_context: &GpuContext,
        resource_id: ResourceId,
        texture: &GpuTexture,
    ) {
        let renderer = &mut self.renderer;
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

        if let Some(id) = registry.get(&resource_id).copied() {
            renderer.textures.replace(id, updated_texture);
        } else {
            let id = renderer.textures.insert(updated_texture);
            registry.insert(resource_id, id);
        }
    }
}
