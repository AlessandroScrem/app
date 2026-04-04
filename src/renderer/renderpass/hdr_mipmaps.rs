use std::default;

use super::*;

#[derive(Default)]
pub struct HdrMipmapsPass {}

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
        &[]
    }
    fn writes(&self) -> &[ResourceId] {
        &[ResourceId::HDRB]
    }

    fn prepare(
        &mut self,
        _asset_mgr: &AssetManager,
        _world: &World,
        _globals: &Globals,
        _selected: Option<Entity>,
        _input: &Input,
        _ctx: &mut RenderContext,
    ) {
    }

    fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        ctx: &mut RenderContext,
        _asset_mgr: &AssetManager,
    ) {
        let device = ctx.device;
        let pipeline = ctx.pip_mgr.get_render_pipeline(PipelineKind::BuildMipmaps);
        let base_view = ctx.gpu_mgr.get_framebuffer_view(FramebufferKind::HdrOpaque);
        let mip_texture = ctx
            .gpu_mgr
            .get_framebuffer_texture(FramebufferKind::HdrOpaque);
        let sampler = ctx
            .gpu_mgr
            .get_framebuffer_sampler(FramebufferKind::HdrOpaque);

        generate_scene_mips(device, encoder, pipeline, base_view, mip_texture, sampler);
        //     let gpu_manager = ctx.gpu_mgr;
        //     let pipeline_manager = ctx.pip_mgr;
        //     let frame_view = &ctx.target;
        //     let device = ctx.device;
        //     let pipeline = pipeline_manager.get_render_pipeline(PipelineKind::BuildMipmaps);

        //     let src_view = ctx.gpu_mgr.get_framebuffer_view(FramebufferKind::HdrOpaque);
        //     let sampler = ctx
        //         .gpu_mgr
        //         .get_framebuffer_sampler(FramebufferKind::HdrOpaque);

        //     let texture = ctx
        //         .gpu_mgr
        //         .get_framebuffer_texture(FramebufferKind::HdrOpaque);
        //     let mip_count = texture.mip_level_count();
        //     assert!(mip_count > 1);

        //     for level in 1..mip_count {
        //         let width = (texture.width() >> level).max(1);
        //         let height = (texture.height() >> level).max(1);

        //         let dst_view = texture.create_view(&wgpu::TextureViewDescriptor {
        //             label: Some(&format!("MipLevel{}", level)),
        //             format: Some(texture.format()),
        //             dimension: Some(wgpu::TextureViewDimension::D2),
        //             aspect: wgpu::TextureAspect::All,
        //             base_mip_level: level,
        //             mip_level_count: Some(1),
        //             base_array_layer: 0,
        //             array_layer_count: Some(1),
        //             ..Default::default()
        //         });

        //         // bind group con mip precedente come input
        //         let bind_group_layout = pipeline.get_bind_group_layout(0);
        //         let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //             label: Some(&format!("MipBindGroup{}", level)),
        //             layout: &bind_group_layout,
        //             entries: &[
        //                 wgpu::BindGroupEntry {
        //                     binding: 0,
        //                     resource: wgpu::BindingResource::TextureView(&src_view),
        //                 },
        //                 wgpu::BindGroupEntry {
        //                     binding: 1,
        //                     resource: wgpu::BindingResource::Sampler(sampler),
        //                 },
        //             ],
        //         });

        //         let src_view = texture.create_view(&wgpu::TextureViewDescriptor {
        //             base_mip_level: level - 1,
        //             mip_level_count: Some(1),
        //             ..Default::default()
        //         });

        //         {
        //             // render pass: src → dst
        //             // Render pass
        //             let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        //                 label: Some(&format!("Hdr Render Mip:{}", level)),
        //                 color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        //                     view: &dst_view,
        //                     resolve_target: None,
        //                     ops: wgpu::Operations {
        //                         load: wgpu::LoadOp::Load,
        //                         store: wgpu::StoreOp::Store,
        //                     },
        //                 })],
        //                 depth_stencil_attachment: None,
        //                 // timestamp_writes: None,
        //                 // occlusion_query_set: None,
        //                 ..Default::default()
        //             });
        //             renderpass.set_pipeline(&pipeline);
        //             renderpass.set_bind_group(0, &bind_group, &[]);
        //             renderpass.draw(0..3, 0..1);
        //         }

        //         // il mip level corrente diventa il source per il prossimo
        //         src_view = &dst_view;
        //     }
        // }
    }
}

fn generate_scene_mips(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    pipeline: &wgpu::RenderPipeline,
    base_view: &wgpu::TextureView, // mip 0 già renderizzata
    mip_texture: &wgpu::Texture,   // texture con mipmap
    sampler: &wgpu::Sampler,
) {
    let mip_count = mip_texture.mip_level_count();
    assert!(mip_count > 1, "Texture must have mip levels");

    // memorizza tutte le texture view dei mip
    let mut mip_views = Vec::with_capacity(mip_count as usize);
    
    mip_views.push(base_view.clone()); // mip 0

    // per ogni mip > 0
    for level in 1..mip_count {
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
                    resource: wgpu::BindingResource::TextureView(&mip_views[level as usize - 1]),
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

        mip_views.push(dst_view); // mantiene il view in vita
    }
}
