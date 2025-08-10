pub fn create() -> impl legion::systems::Runnable {
    use legion::SystemBuilder;

    SystemBuilder::new("imgui")
        .read_resource::<wgpu::TextureView>()
        .write_resource::<wgpu::CommandEncoder>()
        .write_resource::<imgui_wgpu::Renderer>()
        .read_resource::<wgpu::Device>()
        .read_resource::<wgpu::Queue>()
        .read_resource::<imgui::OwnedDrawData>()
        .build(
            |_, _world, (frame_view, encoder, renderer, device, queue, ownned_draw_data), _| {


                let mut pass = {
                    // let encoder = frame.encoder.as_mut().unwrap();
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

                let draw_data = ownned_draw_data.draw_data().unwrap();
                renderer
                    .render(draw_data, queue, device, &mut pass)
                    .unwrap();
            },
        )
}
