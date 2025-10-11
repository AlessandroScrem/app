use app_wgpu::prelude::*;
use winit::event_loop::{ControlFlow, EventLoop};

fn set_default_rust_log() {
    use std::env;
    if env::var("RUST_LOG").is_err() {
        // Usa cfg! per decidere a compile-time il valore
        let default_log = if cfg!(debug_assertions) {
            "app_wgpu=debug" // build debug → livello debug
        } else {
            "app_wgpu=info" // build release → livello info
        };
        unsafe {
            env::set_var("RUST_LOG", default_log);
        }
    }

    env_logger::init();

    // debug!("Debug della mia app");
    // info!("Info della mia app");
    // warn!("Warn della mia app");
    // error!("Error della mia app");
}

fn main() {
    set_default_rust_log();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new_with_size(2400, 1200);
    let _ = event_loop.run_app(&mut app);
}
