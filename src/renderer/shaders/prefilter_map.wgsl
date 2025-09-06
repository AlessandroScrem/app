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
@group(0) @binding(3) var<uniform> roughness: f32;

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


    // remove translation from camera view matrix
    let rot_view = mat4x4<f32>(
        vec4<f32>(camera.view[0].xyz, 0.0), // prima colonna, senza traslazione
        vec4<f32>(camera.view[1].xyz, 0.0), // seconda colonna
        vec4<f32>(camera.view[2].xyz, 0.0), // terza colonna
        vec4<f32>(0.0, 0.0, 0.0, 1.0)       // ultima colonna (nessuna traslazione)
    );

    let clip = camera.proj * rot_view * vec4<f32>(pos, 1.0);

    out.clip_position = clip;
    out.frag_pos = pos;

    return out;
}

/// Fragment shader
///
fn RadicalInverse_VdC(bits: u32) -> f32 {
	var bits_var = bits;
	bits_var = bits_var << 16u | bits_var >> 16u;
	bits_var = (bits_var & 1431655765u) << 1u | (bits_var & 2863311530u) >> 1u;
	bits_var = (bits_var & 858993459u)  << 2u | (bits_var & 3435973836u) >> 2u;
	bits_var = (bits_var & 252645135u)  << 4u | (bits_var & 4042322160u) >> 4u;
	bits_var = (bits_var & 16711935u)   << 8u | (bits_var & 4278255360u) >> 8u;
	return f32(bits_var) * 0.00000000023283064;
} 

fn Hammersley(i: u32, N: u32) -> vec2<f32> {
	return vec2<f32>(f32(i) / f32(N), RadicalInverse_VdC(i));
}

fn ImportanceSampleGGX(Xi: vec2<f32>, N: vec3<f32>, roughness: f32) -> vec3<f32> {
	let a = roughness * roughness;
	let PI: f32 = 3.1415927;
	
  let phi: f32 = 2. * PI * Xi.x;
	let cosTheta = sqrt((1.0 - Xi.y) / (1. + (a * a - 1.0) * Xi.y));
	let sinTheta = sqrt(1.0 - cosTheta * cosTheta);

  // halfway vector in tangent space
	let H = vec3<f32> (
	    cos(phi) * sinTheta,
	    sin(phi) * sinTheta,
	    cosTheta,
    );

  // build an orthonormal basis (tangent, bitangent, normal)
  let up = select(
      vec3<f32>(0.0, 0.0, 1.0),
      vec3<f32>(1.0, 0.0, 0.0),
      abs(N.z) >= 0.999
  );
	
  let tangent = normalize(cross(up, N));
	let bitangent = cross(N, tangent);
	let sampleVec = tangent * H.x + bitangent * H.y + N * H.z;
	return normalize(sampleVec);
}


fn DistributionGGX(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
	let PI: f32 = 3.1415927;
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH = max(dot(N, H), 0.0);
    let NdotH2 = NdotH * NdotH;

    let nom = a2;
    var denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return nom / denom;
}

const SAMPLE_COUNT: u32 = 1024u; // per esempio
fn prefilterEnvironment(N: vec3<f32>, V: vec3<f32>, roughness: f32, saTexel: f32) -> vec3<f32> {
    var prefilteredColor = vec3<f32>(0.0);
    var totalWeight = 0.0;

    for (var i: u32 = 0u; i < SAMPLE_COUNT; i = i + 1u) {
        let Xi = Hammersley(i, SAMPLE_COUNT);          // vec2<f32>
        let H = ImportanceSampleGGX(Xi, N, roughness); // vec3<f32>
        let L = normalize(2.0 * dot(V, H) * H - V);

        let NdotL = max(dot(N, L), 0.0);
        if (NdotL > 0.0) {
            let D = DistributionGGX(N, H, roughness);
            let NdotH = max(dot(N, H), 0.0);
            let HdotV = max(dot(H, V), 0.0);
            let pdf = D * NdotH / (4.0 * HdotV) + 0.0001;

            let saSample = 1.0 / (f32(SAMPLE_COUNT) * pdf + 0.0001);
            let mipLevel = select(0.0, 0.5 * log2(saSample / saTexel), roughness != 0.0);

            let sampleColor = textureSampleLevel(environmentMap, environmentSampler, L, mipLevel).rgb;
            prefilteredColor = prefilteredColor + sampleColor * NdotL;
            totalWeight = totalWeight + NdotL;
        }
    }

    return prefilteredColor / totalWeight;
}

const PI: f32 = 3.1415927;
const RESOLUTION = 512.0; // risoluzione della cubemap sorgente (per faccia)
const SATEXEL  = 4.0 * PI / (6.0 * RESOLUTION * RESOLUTION);
const SAMPLECOUNT = 1024u;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // flip asse Y
    var N = normalize(vec3<f32>(input.frag_pos.x, -input.frag_pos.y, input.frag_pos.z));

    let V = N;

    let color = prefilterEnvironment(N, V, roughness, SATEXEL);

    // debug: visualize normal
    // return vec4<f32>(normalize(input.frag_pos) * 0.5 + 0.5, 1.0);
    return vec4<f32>(color, 1.0);

}