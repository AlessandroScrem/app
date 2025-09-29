use app_wgpu::prelude::*;
use winit::event_loop::{ControlFlow, EventLoop};
use std::thread;
use std::time::Duration;

fn main() {
    // Thread "killer" che aspetta 30s e poi chiude tutto con successo
    thread::spawn(|| {
        thread::sleep(Duration::from_secs(30));
        println!("CI runner: 30s scaduti, termino felicemente.");
        std::process::exit(0);
    });

    // Avvio normale della tua app
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new_with_size(2400, 1200);
    let _ = event_loop.run_app(&mut app);
}
