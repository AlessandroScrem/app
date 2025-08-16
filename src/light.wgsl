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

@group(0) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(0) var<uniform> light: Light;
@group(2) @binding(0) var tex_sampler: sampler;
@group(2) @binding(1) var main_map: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

const BILLBOARD_SIZE = 50.0; // dimensione del billboard in unità di schermo

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
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
    let world_center = vec4<f32>(light.position, 1.0);
    let clip_center = camera.view_proj * world_center;

    // calcola offset in pixel → NDC
    let pixel_size = 2.0 / camera.screen_size; // 2.0 perché NDC va da -1 a 1
    let offset_ndc = quad_pos * BILLBOARD_SIZE * pixel_size;

    // converti offset NDC → clip space (moltiplica per w)
    let offset_clip = vec4<f32>(offset_ndc * clip_center.w, 0.0, 0.0);

    out.clip_position = clip_center + offset_clip;
    out.uv = quad_uv;

    return out;
}

/// Fragment shader
///

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let object_color = textureSample(main_map, tex_sampler, input.uv);

    if (object_color.a < 0.1) {
        discard; // se il colore è trasparente, non disegnare
    }

    return object_color * vec4<f32>(1.0, 1.0, 0.0, 1.0); // giallo; // giallo

}