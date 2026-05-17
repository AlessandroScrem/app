use std::{env, fs, path::Path};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("static_textures.rs");

    let mut output = String::new();

    let textures = [
        ("WHITE_TEXTURE", "assets/core/white.png"),
        ("BLACK_TEXTURE", "assets/core/black.png"),
        ("LIGHTBULB_TEXTURE", "assets/core/lightbulb-icon32.png"),
        ("NORMAL_TEXTURE", "assets/core/empty_normal.png"),
    ];

    for (var_name, path) in textures {
        println!("cargo:rerun-if-changed={}", path);

        let img = image::open(path).expect("Failed to open image").to_rgba8();

        let (width, height) = img.dimensions();
        let data = img.into_raw();

        output.push_str(&format!(
            "const {name}_WIDTH: u32 = {w};\n",
            name = var_name,
            w = width
        ));

        output.push_str(&format!(
            "const {name}_HEIGHT: u32 = {h};\n",
            name = var_name,
            h = height
        ));

        output.push_str(&format!("const {name}: &[u8] = &[\n", name = var_name));

        for chunk in data.chunks(12) {
            output.push_str("    ");
            for byte in chunk {
                output.push_str(&format!("{}, ", byte));
            }
            output.push('\n');
        }

        output.push_str("];\n\n");
    }

    fs::write(dest_path, output).unwrap();
}
