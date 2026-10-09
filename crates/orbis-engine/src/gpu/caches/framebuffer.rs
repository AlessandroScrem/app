use std::collections::HashMap;

use crate::{
    assets::texture_asset::{ColorSpace, SamplerDesc},
    gpu::GpuContextRef,
};

use super::*;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, EnumIter, PartialEq, Eq, Hash)]
pub enum FramebufferKind {
    Hdr,
    OpaqueWithMips,
    EntityId,
    Depth,
}

pub struct Framebuffer {
    texture: GpuTexture,
    bind_group: wgpu::BindGroup,
}

pub struct FramebufferCache {
    framebuffers: Vec<Framebuffer>,
}
impl FramebufferCache {
    pub fn new(
        gpu: &GpuContextRef,
        layouts: &BindgroupLayoutCache,
        width: u32,
        height: u32,
    ) -> Self {
        let framebuffers: Vec<Framebuffer> = FramebufferKind::iter()
            .map(|kind| Self::create(gpu, layouts, kind, width, height))
            .collect();
        Self { framebuffers }
    }

    pub fn resize(
        &mut self,
        gpu: &GpuContextRef,
        layouts: &BindgroupLayoutCache,
        width: u32,
        height: u32,
    ) {
        let framebuffers: Vec<Framebuffer> = FramebufferKind::iter()
            .map(|kind| Self::create(gpu, layouts, kind, width, height))
            .collect();
        self.framebuffers = framebuffers;
    }

    pub fn get_texture(&self, kind: FramebufferKind) -> &wgpu::Texture {
        &self.framebuffers[kind as usize].texture.inner
    }
    pub fn get_sampler(&self, kind: FramebufferKind) -> &wgpu::Sampler {
        &self.framebuffers[kind as usize].texture.sampler
    }
    pub fn get_view(&self, kind: FramebufferKind) -> &wgpu::TextureView {
        &self.framebuffers[kind as usize].texture.view
    }
    pub fn get_view_mips(&self, kind: FramebufferKind) -> &wgpu::TextureView {
        &self.framebuffers[kind as usize].texture.view_mips
    }
    pub fn get_bg(&self, kind: FramebufferKind) -> &wgpu::BindGroup {
        &self.framebuffers[kind as usize].bind_group
    }
    #[allow(unused)]
    pub fn get_map(&self) -> HashMap<FramebufferKind, &GpuTexture> {
        let list = vec![FramebufferKind::Hdr];

        list.iter()
            .map(|k| (k.clone(), &self.framebuffers[*k as usize].texture))
            .collect()
    }
}

impl FramebufferCache {
    fn create(
        gpu: &GpuContextRef,
        layouts: &BindgroupLayoutCache,
        kind: FramebufferKind,
        width: u32,
        height: u32,
    ) -> Framebuffer {
        match kind {
            FramebufferKind::Hdr => {
                let texture = GpuTextureBuilder::from_empty(width, height)
                    .format(ColorSpace::Rgbaf16)
                    .usage(GpuTextureUsage::RenderTarget)
                    .sampler(SamplerDesc::NearestClamp)
                    .label("Hdr texture")
                    .build(gpu);

                let layout = layouts.get(BindgroupLayoutKind::Hdr);
                let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Hdr_bind_group"),
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Sampler(&texture.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&texture.view),
                        },
                    ],
                });

                Framebuffer {
                    texture,
                    bind_group,
                }
            }
            FramebufferKind::OpaqueWithMips => {
                const HDR_MIPS_COUNT: u32 = 8;
                let texture = GpuTextureBuilder::from_empty(width, height)
                    .format(ColorSpace::Rgba8)
                    .with_mips(HDR_MIPS_COUNT)
                    .usage(GpuTextureUsage::SampledTextureStorage)
                    .sampler(SamplerDesc::LinearClampMipmap)
                    .label("Hdr Opaque texture_with_mips")
                    .build(gpu);

                let layout = layouts.get(BindgroupLayoutKind::Hdr);
                let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Hdr_Opaque_bind_group"),
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Sampler(&texture.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&texture.view),
                        },
                    ],
                });

                Framebuffer {
                    texture,
                    bind_group,
                }
            }

            FramebufferKind::EntityId => {
                let texture = GpuTextureBuilder::from_empty(width, height)
                    .format(ColorSpace::Rg32ui)
                    .usage(GpuTextureUsage::EntityId)
                    .sampler(SamplerDesc::NearestClamp)
                    .label("entity_id_texture")
                    .build(gpu);

                let layout = layouts.get(BindgroupLayoutKind::EntityId);
                let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("entity_id_bind_group"),
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Sampler(&texture.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&texture.view),
                        },
                    ],
                });

                Framebuffer {
                    texture,
                    bind_group,
                }
            }
            FramebufferKind::Depth => {
                let texture = GpuTextureBuilder::from_empty(width, height)
                    .format(ColorSpace::Depth32f)
                    .usage(GpuTextureUsage::DepthTarget)
                    .sampler(SamplerDesc::NearestClamp)
                    .label("depth_texture")
                    .build(gpu);

                let layout = layouts.get(BindgroupLayoutKind::Depth);
                let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("depth_bind_group"),
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Sampler(&texture.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&texture.view),
                        },
                    ],
                });

                Framebuffer {
                    texture,
                    bind_group,
                }
            }
        }
    }
}
