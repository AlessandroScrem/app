/// Vertex shader

@group(0) @binding(0) var environmentSampler: sampler;
@group(0) @binding(1) var environmentMap: texture_cube<f32>;
@group(0) @binding(2) var<uniform> view_proj:  mat4x4<f32>;

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
    let clip_position = view_proj * vec4<f32>(pos, 1.0);	

    out.clip_position = clip_position;
    out.frag_pos = pos;

    return out;
}

/// Fragment shader
///

const PI: f32 = 3.1415927;

fn build_onb(n: vec3<f32>) -> mat3x3<f32> {
    let sign = select(-1.0, 1.0, n.z >= 0.0);
    let a = -1.0 / (sign + n.z);
    let b = n.x * n.y * a;

    let tangent = vec3<f32>(
        1.0 + sign * n.x * n.x * a,
        sign * b,
        -sign * n.x
    );

    let bitangent = vec3<f32>(
        b,
        sign + n.y * n.y * a,
        -n.y
    );

    return mat3x3<f32>(tangent, bitangent, n);
}

fn calc_irradiance(dir: vec3<f32>) -> vec3<f32> {
    let N = normalize(dir);
    let onb = build_onb(N);

    var irradiance = vec3<f32>(0.0);
    let sampleDelta = 0.025;
    var nrSamples = 0u;

    for (var phi = 0.0; phi < 2.0 * PI; phi += sampleDelta) {
        for (var theta = 0.0; theta < 0.5 * PI; theta += sampleDelta) {

            let localDir = vec3<f32>(
                sin(theta) * cos(phi),
                sin(theta) * sin(phi),
                cos(theta)
            );

            let sampleVec = normalize(onb * localDir);

            let sampleColor =
                textureSample(environmentMap, environmentSampler, sampleVec).rgb;

            irradiance += sampleColor * cos(theta) * sin(theta);
            nrSamples++;
        }
    }

    return PI * irradiance / f32(nrSamples);
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