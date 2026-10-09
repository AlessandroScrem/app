/// Vertex shader

const MAX_LIGHTS             : u32 = 64;

struct Camera {
    view_pos: vec3<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    screen_size: vec2<f32>,
};

struct Light {
    view_proj: mat4x4<f32>,
    
    color: vec3<f32>,
    directional: u32,

    position: vec3<f32>,
    cast_shadow: u32,
    
    entity_id_low: u32,
    entity_id_high: u32,
}

struct Lights {
    lights: array<Light, MAX_LIGHTS>,
    
    count: u32,
    enabled: u32, 
}

// PerFrame
@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(2) var<uniform> lights: Lights;

// Light Texture
@group(1) @binding(0) var tex_sampler: sampler;
@group(1) @binding(1) var main_map: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) entity_id: vec2<u32>,
};

const BILLBOARD_SIZE = 50.0; // dimensione del billboard in unità di schermo

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_id: u32
) -> VertexOutput {
    var out: VertexOutput;

    // quad locale [-0.5,0.5] in XY
    var quad: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
        vec2<f32>(-0.5, -0.5),
        vec2<f32>( 0.5, -0.5),
        vec2<f32>( 0.5,  0.5),
        vec2<f32>( 0.5,  0.5),
        vec2<f32>(-0.5,  0.5),
        vec2<f32>(-0.5, -0.5),
    );

    // UV nello spazio [0,1]
    var uv: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), // bottom-left
        vec2<f32>(1.0, 0.0), // bottom-right
        vec2<f32>(1.0, 1.0), // top-right
        vec2<f32>(1.0, 1.0), // top-right
        vec2<f32>(0.0, 1.0), // top-left
        vec2<f32>(0.0, 0.0), // bottom-left
    );

    let quad_pos = quad[vertex_index];
    let quad_uv = uv[vertex_index];

    // calcola centro in clip space
    let world_center = vec4<f32>(lights.lights[instance_id].position, 1.0);
    let clip_center = camera.proj * camera.view * world_center;

    // calcola offset in pixel → NDC
    let pixel_size = 2.0 / camera.screen_size; // 2.0 perché NDC va da -1 a 1
    let offset_ndc = quad_pos * BILLBOARD_SIZE * pixel_size;

    // converti offset NDC → clip space (moltiplica per w)
    let offset_clip = vec4<f32>(offset_ndc * clip_center.w, 0.0, 0.0);

    out.clip_position = clip_center + offset_clip;
    out.uv = quad_uv;
    out.entity_id = vec2<u32>(lights.lights[instance_id].entity_id_low, lights.lights[instance_id].entity_id_high);

    return out;
}

/// Fragment shader
///


struct FSOutput {
    @location(0) color : vec4<f32>,
    @location(1) entity_id : vec2<u32>,
}

@fragment
fn fs_main(in: VertexOutput) -> 
    FSOutput {
    var out: FSOutput;

    let object_color = textureSample(main_map, tex_sampler, in.uv);

    if (object_color.a < 0.1) {
        discard; // se il colore è trasparente, non disegnare
    }

    out.color = object_color * vec4<f32>(1.0, 1.0, 1.0, 1.0);
    out.entity_id =  in.entity_id;

    return out; 
}