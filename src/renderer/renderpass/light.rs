pub use super::*;

#[derive(Default)]
pub struct LightPass {
    pub lights: Vec<LightUniform>,
}

impl LightPass {
    pub fn new() -> Self {
        Self::default()
    }
}

impl LightPass {
    fn update_to_gpu(&mut self, ctx: &mut RenderContext) {
        let queue = ctx.queue;
        let gpu_mgr = ctx.gpu_mgr;

        for light in self.lights.iter() {
            queue.write_buffer(&gpu_mgr.light_uniform_buffer, 0, bytemuck::bytes_of(light));
        }
    }
}

impl RenderPass for LightPass {
    fn name(&self) -> &'static str {
        "LightPass"
    }
    fn prepare(
        &mut self,
        _asset_mgr: &AssetManager,
        world: &World,
        _resources: &Resources,
        _camera: &Camera,
        _globals: &Globals,
        _selected: Option<Entity>,
        _input: &Input,
        ctx: &mut RenderContext,
    ) {
        self.lights.clear();

        // -------- Lights --------
        let mut light_query = <(Entity, &LightComponent)>::query();

        for (entity, light) in light_query.iter(world) {
            let data = LightUniform {
                entity_id: entity.as_raw_u64(),
                ..light.data
            };
            self.lights.push(data);
        }

        self.update_to_gpu(ctx);
    }

    fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        ctx: &mut RenderContext,
        _asset_mgr: &AssetManager,
    ) {
        let gpu_manager = ctx.gpu_mgr;
        let pipeline_manager = ctx.pip_mgr;
        let light_manager = ctx.light_mgr;
        let lights = &self.lights;

        // Render pass
        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Light Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &gpu_manager.hdr_frame.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &gpu_manager.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        let pipeline = pipeline_manager.get_render_pipeline(PipelineKind::Light);

        renderpass.set_pipeline(&pipeline);
        renderpass.set_bind_group(0, &gpu_manager.per_frame_bind_group, &[]);
        renderpass.set_bind_group(1, &light_manager.light_texture_bind_group, &[]);

        for _light in lights.iter() {
            renderpass.draw(0..6, 0..1);
        }
    }
}
