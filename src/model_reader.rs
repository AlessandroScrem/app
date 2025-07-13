use gltf::buffer;

#[derive(Default, Debug)]
#[allow(dead_code)]
pub struct Mesh {
    vertices: Vec<cgmath::Vector3<f32>>,
    normals: Vec<cgmath::Vector3<f32>>,
    indices: Vec<u32>,
}


#[allow(dead_code)]
pub fn load_gltf(path: &std::path::Path) -> Result<Vec<Mesh>, Box<dyn std::error::Error>> {
    if path.extension().unwrap_or_default() != "gltf" {
        return Err("File is not a glTF file".into());
    }
    
    let (gltf, buffers, _) = gltf::import(path)?;
    // println!("{:#?}", gltf);

    Ok(read_meshes(&gltf, buffers))
}

fn read_meshes(gltf: &gltf::Document, buffers: Vec<buffer::Data>) -> Vec<Mesh> {
    let mut meshes: Vec<Mesh> = Vec::new();
    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            let mesh = read_mesh(&primitive, buffers.clone());
            meshes.push(mesh);
        }
    }
    meshes
}

fn read_mesh(primitive: &gltf::Primitive, buffers: Vec<buffer::Data>) -> Mesh {
    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
    let positions = reader
        .read_positions()
        .unwrap_or_else(|| panic!("primitives must have the POSITION attribute "));
    let normals = reader
        .read_normals()
        .unwrap_or_else(|| panic!("primitives must have the NORMAL attribute "));
    let indices = reader
        .read_indices()
        .unwrap_or_else(|| panic!("primitives must have the INDICES attribute "));

    Mesh {
        vertices: positions
            .map(|position| cgmath::Vector3::from(position))
            .collect(),
        normals: normals
            .map(|normal| cgmath::Vector3::from(normal))
            .collect(),
        indices: indices.into_u32().collect(),
    }
}

#[allow(dead_code)]
fn print_tree(node: &gltf::Node, depth: i32) {
    for _ in 0..(depth - 1) {
        print!("  ");
    }
    print!(" -");
    print!(" Node {}", node.index());
    print!(" ({})", node.name().unwrap_or("<Unnamed>"));
    println!();

    for child in node.children() {
        print_tree(&child, depth + 1);
    }
}

#[allow(dead_code)]
fn print_meshes(gltf: &gltf::Document, buffers: Vec<buffer::Data>) {
    for mesh in gltf.meshes() {
        println!("Mesh #{}", mesh.index());
        for primitive in mesh.primitives() {
            let index = primitive.indices().unwrap();
            println!(
                "- Primitive #{} Index Count {}",
                primitive.index(),
                index.count()
            );

            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            if let Some(iter) = reader.read_positions() {
                for vertex_position in iter {
                    println!("{:?}", vertex_position);
                }
            }
        }
    }
}
