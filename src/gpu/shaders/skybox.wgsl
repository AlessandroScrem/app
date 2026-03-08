/// Vertex shader

struct Camera {
    view_pos: vec3<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    screen_size: vec2<f32>,
};

struct Globals {
    ibl_enable: u32,
    skybox_enable: u32,
    exposure: f32,
    ibl_intensity: f32,
    selected_entity_id_low: u32,
    selected_entity_id_high: u32,
    tonemap_filter: u32,
    debug: u32,
};

// PerFrame
@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<uniform> globals: Globals;

// Skybox
@group(1) @binding(0) var tex_sampler: sampler;
@group(1) @binding(1) var env_map: texture_cube<f32>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) frag_pos: vec3<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    //local cube [-1 1]
    var box: array<vec3<f32>, 36> = array<vec3<f32>, 36>(
        // +X
        vec3( 1, -1, -1), vec3( 1, -1,  1), vec3( 1,  1,  1),
        vec3( 1, -1, -1), vec3( 1,  1,  1), vec3( 1,  1, -1),
        // -X
        vec3(-1, -1,  1), vec3(-1, -1, -1), vec3(-1,  1, -1),
        vec3(-1, -1,  1), vec3(-1,  1, -1), vec3(-1,  1,  1),

        // +Y (top)
        vec3(-1,  1,  1), vec3( 1,  1,  1), vec3( 1,  1, -1),
        vec3(-1,  1,  1), vec3( 1,  1, -1), vec3(-1,  1, -1),

        // -Y (bottom)
        vec3(-1, -1, -1), vec3( 1, -1, -1), vec3( 1, -1,  1),
        vec3(-1, -1, -1), vec3( 1, -1,  1), vec3(-1, -1,  1),

        // +Z
        vec3(-1, -1,  1), vec3(-1,  1,  1), vec3( 1,  1,  1),
        vec3(-1, -1,  1), vec3( 1,  1,  1), vec3( 1, -1,  1),
        // -Z
        vec3( 1, -1, -1), vec3( 1,  1, -1), vec3(-1,  1, -1),
        vec3( 1, -1, -1), vec3(-1,  1, -1), vec3(-1, -1, -1)

    );

    // remove translation from camera view matrix
    let rot_view = mat4x4<f32>(
        vec4<f32>(camera.view[0].xyz, 0.0), // prima colonna, senza traslazione
        vec4<f32>(camera.view[1].xyz, 0.0), // seconda colonna
        vec4<f32>(camera.view[2].xyz, 0.0), // terza colonna
        vec4<f32>(0.0, 0.0, 0.0, 1.0)       // ultima colonna (nessuna traslazione)
    );

    let pos = box[vertex_index];

    let clip_position = camera.proj * rot_view * vec4<f32>(pos, 1.0);	

    out.clip_position = clip_position.xyww;
    out.frag_pos = pos;

    return out;
}

/// Fragment shader
///
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    
    // flip asse X
    let dir = vec3<f32>(-input.frag_pos.x, input.frag_pos.y, input.frag_pos.z);

    var color = textureSampleLevel(env_map, tex_sampler, dir, 0.0).rgb;
    color *= globals.ibl_intensity;

    return vec4<f32>(color, 1.0); 

}