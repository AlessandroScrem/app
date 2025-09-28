use app_wgpu::prelude::*;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new_with_size(2400, 1200);
    let _ = event_loop.run_app(&mut app);

}