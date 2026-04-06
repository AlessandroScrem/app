use log::warn;

use super::*;
use crate::renderer;

#[derive(Default)]
pub struct HdrMipmapsPass {
    mips_enable: bool,
}

impl HdrMipmapsPass {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RenderPass for HdrMipmapsPass {
    fn name(&self) -> &'static str {
        "HdrMipmapsPass"
    }

    fn reads(&self) -> &[ResourceId] {
        &[ResourceId::HDRA]
    }
    fn writes(&self) -> &[ResourceId] {
        &[ResourceId::HDRB]
    }

    fn prepare(
        &mut self,
        _asset_mgr: &AssetManager,
        _world: &World,
        globals: &Globals,
        _selected: Option<Entity>,
        _input: &Input,
        _ctx: &mut RenderContext,
    ) {
        self.mips_enable = globals.mips_cs;
    }

    fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        ctx: &mut RenderContext,
        _asset_mgr: &AssetManager,
    ) {
        let device = ctx.device;
        let src_texture = ctx.gpu_mgr.get_framebuffer_texture(FramebufferKind::Hdr);
        let mip_texture = ctx
            .gpu_mgr
            .get_framebuffer_texture(FramebufferKind::HdrOpaque);

        // copy_to_mip0(encoder, src_texture, mip_texture);

        let pipeline = ctx
            .pip_mgr
            .get_compute_pipeline(renderer::CsPipelineKind::CopyToMipmaps01);

        copy_to_mip01(device, encoder, pipeline, src_texture, mip_texture);

        // create with compute pipeline
        if self.mips_enable {
            let cs_pipeline = ctx
                .pip_mgr
                .get_compute_pipeline(renderer::CsPipelineKind::BuildMipmaps);
            let bg_layout = ctx.gpu_mgr.get_layout(LayoutKind::CsMipmaps);

            compute_mipmaps(device, encoder, cs_pipeline, mip_texture, bg_layout);
        }
        // create with render pipeline
        else {
            let pipeline = ctx.pip_mgr.get_render_pipeline(PipelineKind::BuildMipmaps);
            let sampler = ctx
                .gpu_mgr
                .get_framebuffer_sampler(FramebufferKind::HdrOpaque);

            generate_scene_mips(device, encoder, pipeline, mip_texture, sampler);
        }
    }
}

fn copy_to_mip0(
    encoder: &mut wgpu::CommandEncoder,
    src_texture: &wgpu::Texture, // texture src già renderizzata
    mip_texture: &wgpu::Texture, // texture dst con mipmap
) {
    assert_eq!(
        src_texture.format(),
        mip_texture.format(),
        "Textures must have same format"
    );

    encoder.copy_texture_to_texture(
        wgpu::TexelCopyTextureInfo {
            texture: src_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyTextureInfo {
            texture: mip_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        src_texture.size(),
    );
}

fn copy_to_mip01(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    pipeline: &wgpu::ComputePipeline,
    src_texture: &wgpu::Texture, // texture src già renderizzata
    mip_texture: &wgpu::Texture, // texture dst con mipmap
) {
    if mip_texture.mip_level_count() == 1 {
        warn!("Texture must have mip levels");
        return;
    }

    let src_view = src_texture.create_view(&wgpu::TextureViewDescriptor {
        mip_level_count: Some(1),
        ..Default::default()
    });

    let dst_view_mip0 = mip_texture.create_view(&wgpu::TextureViewDescriptor {
        base_mip_level: 0,
        mip_level_count: Some(1),
        ..Default::default()
    });

    let dst_view_mip1 = mip_texture.create_view(&wgpu::TextureViewDescriptor {
        base_mip_level: 1,
        mip_level_count: Some(1),
        ..Default::default()
    });

    // ogni thread processa 2x2 pixel
    let dispatch_x = (mip_texture.width() + 15) / 16;
    let dispatch_y = (mip_texture.height() + 15) / 16;

    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute pass mip01 creation"),
            ..Default::default()
        });
        let bind_group_layout = pipeline.get_bind_group_layout(0);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("MipLevel01"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&src_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&dst_view_mip0),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&dst_view_mip1),
                },
            ],
        });

        compute_pass.set_pipeline(pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);

        compute_pass.dispatch_workgroups(dispatch_x, dispatch_y, 1);
    }
}

fn compute_mipmaps(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    pipeline: &wgpu::ComputePipeline,
    mip_texture: &wgpu::Texture, // texture con mips
    bg_layout: &wgpu::BindGroupLayout,
) {
    if mip_texture.mip_level_count() == 1 {
        warn!("Texture must have mip levels");
        return;
    }

    let mut src_view = mip_texture.create_view(&wgpu::TextureViewDescriptor {
        base_mip_level: 1,
        mip_level_count: Some(1),
        ..Default::default()
    });

    let dispatch_x = mip_texture.width().div_ceil(16);
    let dispatch_y = mip_texture.height().div_ceil(16);

    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute pass mips creation"),
            ..Default::default()
        });

        compute_pass.set_pipeline(pipeline);

        for mip in 2..mip_texture.mip_level_count() {
            let dst_view = mip_texture.create_view(&wgpu::TextureViewDescriptor {
                base_mip_level: mip,
                mip_level_count: Some(1),
                ..Default::default()
            });
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("MipLevel{}", mip)),
                layout: bg_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&src_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&dst_view),
                    },
                ],
            });
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(dispatch_x, dispatch_y, 1);

            src_view = dst_view;
        }
    }
}

// render mipmaps
fn generate_scene_mips(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    pipeline: &wgpu::RenderPipeline,
    mip_texture: &wgpu::Texture, // texture dst con mipmap
    sampler: &wgpu::Sampler,
) {
    if mip_texture.mip_level_count() == 1 {
        warn!("Texture must have mip levels");
        return;
    }

    let mut src_view = mip_texture.create_view(&wgpu::TextureViewDescriptor {
        base_mip_level: 1,
        mip_level_count: Some(1),
        ..Default::default()
    });

    // per ogni mip > 0
    for level in 2..mip_texture.mip_level_count() {
        let dst_view = mip_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("MipLevel{}", level)),
            base_mip_level: level,
            mip_level_count: Some(1),
            ..Default::default()
        });

        // bind group con mip precedente come input
        let bind_group_layout = pipeline.get_bind_group_layout(0);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("MipBindGroup{}", level)),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&src_view),
                },
            ],
        });

        // render pass sul mip corrente
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("RenderMip{}", level)),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &dst_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            rpass.set_pipeline(pipeline);
            rpass.set_bind_group(0, &bind_group, &[]);
            rpass.draw(0..3, 0..1); // fullscreen triangle
        }
        src_view = dst_view;
    }
}
