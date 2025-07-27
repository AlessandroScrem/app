/// Vertex shader

struct Camera {
    view_pos: vec3<f32>,
    view_proj: mat4x4<f32>,
};

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
};


@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {

    var out: VertexOutput;

    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.world_pos = model.position;
    out.normal =  normalize(model.normal);
    out.color = model.color;

    return out;
}

/// Fragment shader
///

const LIGHT_DIRECTION: vec3<f32> = vec3<f32>(0.0, 0.0, -1.0);
const LIGHT_COLOR: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0);
const AMBIENT_COLOR: vec3<f32> = vec3<f32>(0.2, 0.2, 0.2);
const MATERIAL_SHININESS: f32 = 4.0;
const MATERIAL_SPECULAR: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0);

/// Calcola il colore secondo il modello Blinn-Phong
fn blinn_phong(material_color: vec3<f32>, vert_normal: vec3<f32>, frag_pos: vec3<f32>) -> vec3<f32> {
    let light_dir = normalize(-LIGHT_DIRECTION);
    let view_dir = normalize(camera.view_pos - frag_pos);
    let halfway_dir = normalize(light_dir + view_dir);
    let normal = normalize(vert_normal);

    // Ambient
    let ambient = AMBIENT_COLOR * material_color;

    // Diffuse
    let diff = max(dot(normal, light_dir), 0.0);
    let diffuse = material_color  * diff * LIGHT_COLOR;

    // Specular
    let spec = pow(max(dot(normal, halfway_dir), 0.0), MATERIAL_SHININESS);
    let specular = MATERIAL_SPECULAR * spec * LIGHT_COLOR;

    return clamp(ambient + diffuse + specular, vec3<f32>(0.0), vec3<f32>(1.0));
}

fn debug_color(normal: vec3<f32>) -> vec3<f32> {
    // Le normali vanno da [-1, 1] → mappiamole a [0, 1] per visualizzarle
    return normal * 0.5 + vec3<f32>(0.5);
}


@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let color = blinn_phong(input.color, input.normal, input.world_pos);
    // let color = debug_color(input.normal); // Per visualizzare le normali
    
    return vec4<f32>(color, 1.0);
}