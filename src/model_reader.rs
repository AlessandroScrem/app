use gltf::buffer;

#[derive(Default)]
#[allow(dead_code)]
pub struct Mesh {
    vertices: Vec<cgmath::Point3<f32>>,
    indices: Vec<u32>,
}

pub fn load_gltf(path: &str) -> Result<Vec<Mesh>, Box<dyn std::error::Error>> {
    let (gltf, buffers, _) = gltf::import(path)?;

    // println!("{:#?}", gltf);

    for scene in gltf.scenes() {
        print!("Scene {}", scene.index());
        print!(" ({})", scene.name().unwrap_or("<Unnamed>"));
        println!();
        for node in scene.nodes() {
            print_tree(&node, 1);
        }
    }

    print_meshes(&gltf, buffers.clone());

    Ok(read_mesh(&gltf, buffers))
}

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

fn read_mesh(gltf: &gltf::Document, buffers: Vec<buffer::Data>) -> Vec<Mesh> {
    let mut meshes: Vec<Mesh> = Vec::new();
    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            let positions = reader.read_positions().unwrap();
            let indices = reader.read_indices().unwrap();
            meshes.push(Mesh {
                vertices: positions
                    .map(|v| cgmath::Point3::new(v[0], v[1], v[2]))
                    .collect(),
                indices: indices.into_u32().collect(),
            });
        }
    }
    meshes
}

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
