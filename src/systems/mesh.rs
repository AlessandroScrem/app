use crate::{
    renderer::pipeline_manager::PipelineManager, resources::gpu_manager::GPUResourceManager,
};

pub fn create() -> impl legion::systems::Runnable {
    use legion::SystemBuilder;

    SystemBuilder::new("render mesh")
        .read_resource::<GPUResourceManager>()
        .read_resource::<PipelineManager>()
        .read_resource::<wgpu::Device>()
        .write_resource::<wgpu::Surface>()
        .write_resource::<wgpu::Queue>()
        .build(
            |_, _world, (gpu_resource_manager, pipeline_manager, device, surface, queue), _| {
                let output = match surface.get_current_texture() {
                    Ok(out) => out,
                    Err(_) => return,
                };

                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("mesh"),
                });

                {
                    let clear_color = wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    };

                    let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(clear_color),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    let render_pipeline = match pipeline_manager.get_render_pipeline("default") {
                        Some(pip) => pip,
                        None => return,
                    };

                    renderpass.set_pipeline(render_pipeline);
                    renderpass.set_bind_group(0, &gpu_resource_manager.camera_bind_group, &[]);
                    renderpass.set_vertex_buffer(0, gpu_resource_manager.vertex_buffer.0.slice(..));
                    renderpass.draw(0..3, 0..1);
                }

                queue.submit([encoder.finish()]);
                // TODO: check if it's mandatory
                // self.window.pre_present_notify();
                output.present();
            },
        )
}
