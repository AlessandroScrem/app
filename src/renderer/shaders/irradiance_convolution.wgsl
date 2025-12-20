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

    let pos = box[vertex_index];
    let clip_position = camera.proj * camera.view * vec4<f32>(pos, 1.0);	

    out.clip_position = clip_position;
    out.frag_pos = pos;

    return out;
}

/// Fragment shader
///

const PI: f32 = 3.1415927;


fn calc_irradiance(dir: vec3<f32>) ->vec3<f32> {
    let N = normalize(dir);
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
    return irradiance;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // direzione dal centro della cubemap
    let d0 = normalize(input.frag_pos);

    // Flip asse Y
    let dir  = vec3<f32>(d0.x, -d0.y, d0.z);

    //debug dir result
    // let color = textureSample(environmentMap, environmentSampler, dir).rgb;

    let color = calc_irradiance(dir);
    return vec4<f32>(color, 1.0);
}