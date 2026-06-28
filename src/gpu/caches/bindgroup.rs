use crate::assets::texture_asset::{ColorSpace, SamplerDesc};

use super::*;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum BindgroupKind {
    Camera,
    Perframe,
    LightTexture,
    PbrMap,
    Skybox,
    SkyboxBlur,
    ShadowMap,
    ShadowMapCreate,
}

pub struct BindgroupCache {
    bg: Vec<wgpu::BindGroup>,
}

impl BindgroupCache {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        buffer_cache: &BufferCache,
        framebuffer_cache: &FramebufferCache,
        layouts: &BindgroupLayoutCache,
    ) -> Self {
        let bg: Vec<wgpu::BindGroup> = BindgroupKind::iter()
            .map(|kind| {
                Self::create(
                    device,
                    queue,
                    buffer_cache,
                    &framebuffer_cache,
                    layouts,
                    kind,
                )
            })
            .collect();
        Self { bg }
    }
    pub fn get(&self, kind: BindgroupKind) -> &wgpu::BindGroup {
        &self.bg[kind as usize]
    }
    pub fn get_mut(&mut self, kind: BindgroupKind) -> &mut wgpu::BindGroup {
        &mut self.bg[kind as usize]
    }
}

impl BindgroupCache {
    fn create(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        buffer_cache: &BufferCache,
        framebuffer_cache: &FramebufferCache,
        layouts: &BindgroupLayoutCache,
        kind: BindgroupKind,
    ) -> wgpu::BindGroup {
        match kind {
            BindgroupKind::Camera => device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: layouts.get(BindgroupLayoutKind::Camera),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer_cache.get(BufferKind::Camera).as_entire_binding(),
                }],
                label: Some("Camera Bind Group"),
            }),
            BindgroupKind::Perframe => {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: layouts.get(BindgroupLayoutKind::PerFrame),
                    entries: &[
                        // Camera
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: buffer_cache.get(BufferKind::Camera).as_entire_binding(),
                        },
                        // GLobals
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: buffer_cache.get(BufferKind::Globals).as_entire_binding(),
                        },
                        // Light
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: buffer_cache.get(BufferKind::Light).as_entire_binding(),
                        },
                    ],
                    label: Some("PerFrame Bind Group"),
                })
            }
            BindgroupKind::LightTexture => {
                let texture =
                    GpuTextureBuilder::from_static(&static_textures::LIGHTBULB_STATIC_TEXTURE)
                        .sampler(SamplerDesc::LinearRepeat)
                        .build(device, Some(queue));

                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: layouts.get(BindgroupLayoutKind::LightTexture),
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
                    label: Some("light texture_bind_group"),
                })
            }
            BindgroupKind::PbrMap => {
                let scene_view = framebuffer_cache.get_view(FramebufferKind::OpaqueWithMips);
                let scene_sampler = framebuffer_cache.get_sampler(FramebufferKind::OpaqueWithMips);

                let texture =
                    GpuTextureBuilder::from_static(&static_textures::WHITE_STATIC_TEXTURE)
                        .build(device, Some(queue));

                let cube = GpuTextureBuilder::from_static(&static_textures::WHITE_STATIC_TEXTURE)
                    .dimension(Dimension::Cube)
                    .format(ColorSpace::Rgba8)
                    .usage(GpuTextureUsage::SampledTexture)
                    .sampler(SamplerDesc::LinearRepeat)
                    .label("Cube white texture")
                    .build(device, Some(queue));

                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: layouts.get(BindgroupLayoutKind::PbrMaps),
                    entries: &[
                        // sampler
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Sampler(&cube.sampler),
                        },
                        // irradiance texture
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&cube.view),
                        },
                        // prefiltered texture
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(&cube.view),
                        },
                        // brdf_lut texture
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::TextureView(&texture.view),
                        },
                        // opaque scene sampler
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: wgpu::BindingResource::Sampler(&scene_sampler),
                        },
                        // opaque scene texture
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: wgpu::BindingResource::TextureView(&scene_view),
                        },
                    ],
                    label: Some("Fake Ibl Bind Group"),
                })
            }
            BindgroupKind::Skybox => {
                let cube = GpuTextureBuilder::from_static(&static_textures::WHITE_STATIC_TEXTURE)
                    .dimension(Dimension::Cube)
                    .format(ColorSpace::Rgba8)
                    .usage(GpuTextureUsage::SampledTexture)
                    .sampler(SamplerDesc::LinearRepeat)
                    .label("Cube white texture")
                    .build(device, Some(queue));

                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: layouts.get(BindgroupLayoutKind::Skybox),
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Sampler(&cube.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&cube.view),
                        },
                    ],
                    label: Some("skybox_bind_group"),
                })
            }
            BindgroupKind::SkyboxBlur => {
                let cube = GpuTextureBuilder::from_static(&static_textures::WHITE_STATIC_TEXTURE)
                    .dimension(Dimension::Cube)
                    .format(ColorSpace::Rgba8)
                    .usage(GpuTextureUsage::SampledTexture)
                    .sampler(SamplerDesc::LinearRepeat)
                    .label("Cube white texture")
                    .build(device, Some(queue));

                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: layouts.get(BindgroupLayoutKind::Skybox),
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Sampler(&cube.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&cube.view),
                        },
                    ],
                    label: Some("skybox_blur_bind_group"),
                })
            }
            BindgroupKind::ShadowMap => {
                let view = framebuffer_cache.get_view(FramebufferKind::ShadowMap);
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: layouts.get(BindgroupLayoutKind::ShadowMap),
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(view),
                    }],
                    label: Some("ShadowMap_bind_group"),
                })
            }
            BindgroupKind::ShadowMapCreate => {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: layouts.get(BindgroupLayoutKind::ShadowMapCreate),
                    entries: &[
                        // Light
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: buffer_cache.get(BufferKind::Light).as_entire_binding(),
                        },
                    ],
                    label: Some("ShadowMapCreate_bind_group"),
                })
            }
        }
    }
}
