/// Vertex shader

struct Camera {
    view_pos: vec3<f32>,
    view_proj: mat4x4<f32>,
    screen_size: vec2<f32>,
};

struct Light {
    color: vec3<f32>,
    directional: u32,
    position: vec3<f32>,
    cast_shadow: u32,
}

struct Model {
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
    @location(3) uv: vec2<f32>,
};

@group(0) @binding(0) var<uniform> camera: Camera;
@group(2) @binding(0) var<uniform> model: Model;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
    @location(3) uv: vec2<f32>,
};


@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {

    var out: VertexOutput;
    let world_position = model.model * vec4<f32>(vertex.position, 1.0);

    out.clip_position = camera.view_proj * world_position;
    out.world_pos = world_position.xyz;
    out.normal = normalize((model.normal_matrix * vec4<f32>(vertex.normal, 0.0)).xyz);
    out.uv =  vertex.uv;
    out.color = vertex.color;

    return out;
}

/// Fragment shader
///

const LIGHT_TARGET: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);
const AMBIENT_COLOR: vec3<f32> = vec3<f32>(0.2, 0.2, 0.2);
const MATERIAL_SHININESS: f32 = 4.0;
const MATERIAL_SPECULAR: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0);

/// Calcola il colore secondo il modello Blinn-Phong
fn blinn_phong(material_color: vec3<f32>, vert_normal: vec3<f32>, frag_pos: vec3<f32>) -> vec3<f32> {
    let light_dir = normalize(light.position - LIGHT_TARGET);
    let view_dir = normalize(camera.view_pos - frag_pos);
    let halfway_dir = normalize(light_dir + view_dir);
    let normal = normalize(vert_normal);

    // Ambient
    let ambient = AMBIENT_COLOR * material_color;

    // Diffuse
    let diff = max(dot(normal, light_dir), 0.0);
    let diffuse = material_color  * diff * light.color;

    // Specular
    let spec = pow(max(dot(normal, halfway_dir), 0.0), MATERIAL_SHININESS);
    let specular = MATERIAL_SPECULAR * spec * light.color;

    return clamp(ambient + diffuse + specular, vec3<f32>(0.0), vec3<f32>(1.0));
}

fn debug_normal(normal: vec3<f32>) -> vec3<f32> {
    // rimap da [-1, 1] → a [0, 1] per visualizzarle
    return normal * 0.5 + vec3<f32>(0.5);
}
fn debug_uv(uv: vec2<f32>) -> vec3<f32> {
    // rimap da [-1, 1] → a [0, 1] per visualizzarle
    return vec3<f32>(uv * 0.5 + vec2<f32>(0.5), 0.0);
}

@group(1) @binding(0) var tex_sampler: sampler;
@group(1) @binding(1) var main_map: texture_2d<f32>;
@group(1) @binding(2) var normal_map: texture_2d<f32>;
@group(1) @binding(3) var roughness_map: texture_2d<f32>;

@group(3) @binding(0) var<uniform> light: Light;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let object_color = textureSample(main_map, tex_sampler, input.uv).rgb;
    let object_normal = textureSample(normal_map, tex_sampler, input.uv).rgb;

    let color = blinn_phong(object_color, input.normal, input.world_pos);

    return vec4<f32>(color, 1.0);
}