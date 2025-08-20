/// Vertex shader

struct Camera {
    view_pos: vec3<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    screen_size: vec2<f32>,
};

struct Light {
    color: vec3<f32>,
    directional: u32,
    position: vec3<f32>,
    cast_shadow: u32,
}

@group(0) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(0) var tex_sampler: sampler;
@group(1) @binding(1) var env_map: texture_cube<f32>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) frag_pos: vec3<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    // quad locale [-0.5,0.5] in XY
    var skybox: array<vec3<f32>, 36> = array<vec3<f32>, 36>(
        vec3<f32>(-1.0, -1.0, -1.0),
        vec3<f32>( 1.0,  1.0, -1.0),
        vec3<f32>( 1.0, -1.0, -1.0),
        vec3<f32>( 1.0,  1.0, -1.0),
        vec3<f32>(-1.0, -1.0, -1.0),
        vec3<f32>(-1.0,  1.0, -1.0),

        vec3<f32>(-1.0, -1.0,  1.0),
        vec3<f32>( 1.0, -1.0,  1.0),
        vec3<f32>( 1.0,  1.0,  1.0),
        vec3<f32>( 1.0,  1.0,  1.0),
        vec3<f32>(-1.0,  1.0,  1.0),
        vec3<f32>(-1.0, -1.0,  1.0),

        vec3<f32>(-1.0,  1.0,  1.0),
        vec3<f32>(-1.0,  1.0, -1.0),
        vec3<f32>(-1.0, -1.0, -1.0),
        vec3<f32>(-1.0, -1.0, -1.0),
        vec3<f32>(-1.0, -1.0,  1.0),
        vec3<f32>(-1.0,  1.0,  1.0),

        vec3<f32>(1.0,  1.0,  1.0),
        vec3<f32>(1.0, -1.0, -1.0),
        vec3<f32>(1.0,  1.0, -1.0),
        vec3<f32>(1.0, -1.0, -1.0),
        vec3<f32>(1.0,  1.0,  1.0),
        vec3<f32>(1.0, -1.0,  1.0),
        
        vec3<f32>(-1.0, -1.0, -1.0),
        vec3<f32>( 1.0, -1.0, -1.0),
        vec3<f32>( 1.0, -1.0,  1.0),
        vec3<f32>( 1.0, -1.0,  1.0),
        vec3<f32>(-1.0, -1.0,  1.0),
        vec3<f32>(-1.0, -1.0, -1.0),
        
        vec3<f32>(-1.0,  1.0, -1.0),
        vec3<f32>( 1.0,  1.0,  1.0),
        vec3<f32>( 1.0,  1.0, -1.0),
        vec3<f32>( 1.0,  1.0,  1.0),
        vec3<f32>(-1.0,  1.0, -1.0),
        vec3<f32>(-1.0,  1.0,  1.0),

    );

    // remove translation from camera view matrix
    let rot_view = mat4x4<f32>(
        vec4<f32>(camera.view[0].xyz, 0.0), // prima colonna, senza traslazione
        vec4<f32>(camera.view[1].xyz, 0.0), // seconda colonna
        vec4<f32>(camera.view[2].xyz, 0.0), // terza colonna
        vec4<f32>(0.0, 0.0, 0.0, 1.0)       // ultima colonna (nessuna traslazione)
    );

    let pos = skybox[vertex_index];

    let clip_position = camera.proj * rot_view * vec4<f32>(pos, 1.0);	

    out.clip_position = clip_position.xyww;
    out.frag_pos = pos;

    return out;
}

/// Fragment shader
///

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let color: vec3<f32> = textureSampleLevel(env_map, tex_sampler, input.frag_pos, 0.0).rgb;

    return vec4<f32>(color, 1.0); 

}