use legion::system;
#[system]
pub fn execute_start(cmd: &mut legion::systems::CommandBuffer) {
    cmd.exec_mut(|_world, resources| {
        let (frame, view, encoder) = {
            let device = resources.get::<wgpu::Device>().unwrap();
            let surface = resources.get::<wgpu::Surface>().unwrap();
            let frame = surface
                .get_current_texture()
                .expect("Failed to get current texture");
            let view = frame.texture.create_view(&Default::default());
            let encoder = device.create_command_encoder(&Default::default());
            (frame, view, encoder)
        };

        resources.insert(encoder);
        resources.insert(frame);
        resources.insert(view);
        
    });
}

#[system]
pub fn execute_finish(cmd: &mut legion::systems::CommandBuffer) {
    cmd.exec_mut(|_world, resources| {
        
        let frame = resources.remove::<wgpu::SurfaceTexture>().unwrap();
        let encoder = resources.remove::<wgpu::CommandEncoder>().unwrap();
        let queue = resources.get::<wgpu::Queue>().unwrap();
        
        queue.submit([encoder.finish()]);
        frame.present();
    });
}
