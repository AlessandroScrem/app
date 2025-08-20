use image::{ImageFormat, ImageReader};

fn main() {
    println!("Hello hdr loader");

    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/core/clarens_night_02_2k.hdr"
    );

    let image = ImageReader::open(path).unwrap().decode().unwrap();
    println!("Read image : {:?}", image.color());
    println!("Size : [{} x {}]", image.width(), image.height());
    
    let format = ImageFormat::from_path(path);
    
    println!("ImageFormat : {:?}", format.unwrap());
}
