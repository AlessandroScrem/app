use std::path::Path;

fn main() {
    let paths = vec![
        Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/skybox/right.png"
        )),
        Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/skybox/left.png"
        )),
        Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/skybox/top.png"
        )),
        Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/skybox/bottom.png"
        )),
        Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/skybox/front.png"
        )),
        Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/skybox/back.png"
        )),
    ];

    let start = std::time::Instant::now();
    let buffer = std::fs::read(paths[0]).expect("unable to read file");
    let elapsed = start.elapsed();

    println!("Read {} bytes in {:?}", buffer.len(), elapsed);

    let start = std::time::Instant::now();
    let image = image::load_from_memory(&buffer).unwrap();

    let elapsed = start.elapsed();
    println!(
        "Read from memory{} bytes in {:?}",
        image.as_bytes().len(),
        elapsed
    );

    let start = std::time::Instant::now();
    let image = image::ImageReader::open(paths[0])
        .unwrap()
        .decode()
        .unwrap();

    let elapsed = start.elapsed();
    println!(
        "Read fromImageReader {} bytes in {:?}",
        image.as_bytes().len(),
        elapsed
    );
}
