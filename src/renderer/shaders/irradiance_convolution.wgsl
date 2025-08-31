/// Vertex shader

struct Camera {
    view_pos: vec3<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    screen_size: vec2<f32>,
};

@group(0) @binding(0) var environmentSampler: sampler;
@group(0) @binding(1) var environmentMap: texture_cube<f32>;
@group(0) @binding(2) var<uniform> camera: Camera;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) frag_pos: vec3<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    //local cube [-1 1]
    var box: array<vec3<f32>, 36> = array<vec3<f32>, 36>(
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

    let pos = box[vertex_index];

    let clip_position = camera.proj * rot_view * vec4<f32>(pos, 1.0);	

    out.clip_position = clip_position.xyww;
    out.frag_pos = pos;

    return out;
}

/// Fragment shader
///

const PI: f32 = 3.1415927;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let N = normalize(input.frag_pos);
    var irradiance = vec3<f32>(0.0, 0.0, 0.0);

    // tangent space calculation from origin point
    var up    = vec3<f32>(0.0, 1.0, 0.0);
    var right = normalize(cross(up, N));
    up        = normalize(cross(N, right));

    let sampleDelta = 0.025;
    var nrSamples = 0u;
    for (var phi = 0.0; phi < 2.0 * PI; phi += sampleDelta) {
        for (var theta = 0.0; theta < 0.5 * PI; theta += sampleDelta) {
            // spherical to cartesian (in tangent space)
            let tangentSample = vec3<f32>(
                sin(theta) * cos(phi),
                sin(theta) * sin(phi),
                cos(theta)
            );
            // tangent space to world
            let sampleVec = tangentSample.x * right + tangentSample.y * up + tangentSample.z * N;

            let sampleColor = textureSample(environmentMap, environmentSampler, sampleVec).rgb;
            irradiance += sampleColor * cos(theta) * sin(theta);
            nrSamples++;
        }
    }
    irradiance = PI * irradiance * (1.0 / f32(nrSamples));
    let color = irradiance;

    return vec4<f32>(color, 1.0);

}