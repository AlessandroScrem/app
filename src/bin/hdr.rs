
fn main() {
    println!("Hello hdr loader");

    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/core/clarens_night_02_2k.hdr"
    );
    
    // reading from buffer
    let buffer = std::fs::read(path).expect("unable to read file");
    let image = image::load_from_memory(&buffer).unwrap();
    
    // readimg from path
    // use image::{GenericImageView, ImageFormat, ImageReader};
    // let image = ImageReader::open(path).unwrap().decode().unwrap();
    // println!("Read image : {:?}", image.color());
    // println!("Size : [{} x {}]", image.width(), image.height());
    // let format = ImageFormat::from_path(path);
    // println!("ImageFormat : {:?}", format.unwrap());

    println!("DynamicImage variant: {:?}", image.color());

    let pixel = image.as_rgb32f().unwrap().get_pixel(1228, 385);
    println!("Pixel rgb 32f: {:?}", pixel.0);

    let image_rgba32f = image.to_rgba32f();
    let pixel = image_rgba32f.get_pixel(1228, 385);
    println!("Pixel rgba 32f: {:?}", pixel.0);
}
