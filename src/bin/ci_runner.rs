use app_wgpu::Engine;

use std::thread;
use std::time::Duration;

fn main() ->Result<(), Box<dyn std::error::Error>>{
    // Thread "killer" che aspetta 30s e poi chiude tutto con successo
    thread::spawn(|| {
        thread::sleep(Duration::from_secs(30));
        println!("CI runner: app stop gracefully after 30 secs.");
        std::process::exit(0);
    });
    
    // Avvio normale della tua app
    println!("CI runner: app  started ... \n\t will be stopped after 30 secs..\n");

     Engine::new_with_size(2400, 1200).run()
}
