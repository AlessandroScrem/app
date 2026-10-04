/// Vertex shader

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex 
fn vs_main(@builtin(vertex_index) vertex_index: u32)  -> VertexOutput {
    var out: VertexOutput;
    // quad locale [-1.0,1.0] in clip space
    var quad: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
    );


    // UV nello spazio [0,1]
    var uv: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0), // bottom-left
        vec2<f32>(1.0, 1.0), // bottom-right
        vec2<f32>(1.0, 0.0), // top-right
        vec2<f32>(1.0, 0.0), // top-right
        vec2<f32>(0.0, 0.0), // top-left
        vec2<f32>(0.0, 1.0), // bottom-left
    );

    out.clip_position = vec4<f32>( quad[vertex_index], 0.0, 1.0); // scala a schermo pieno
    out.uv =  uv[vertex_index];

    return out;
} 

/// Fragment shader
///

fn GeometrySchlickGGX_ForIBL(NdotV: f32, roughness: f32) -> f32 {
	let a = roughness;
	let k = a * a / 2.0;
	let nom: f32 = NdotV;
	let denom: f32 = NdotV * (1.0 - k) + k;
	return nom / denom;
} 

fn GeometrySmith_ForIBL(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
	let NdotV: f32 = max(dot(N, V), 0.0);
	var NdotL: f32 = max(dot(N, L), 0.0);
	let ggx2: f32 = GeometrySchlickGGX_ForIBL(NdotV, roughness);
	let ggx1: f32 = GeometrySchlickGGX_ForIBL(NdotL, roughness);
	return ggx1 * ggx2;
}

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

fn IntegrateBRDF(NdotV: f32, roughness: f32) -> vec2<f32> {
    var V = vec3<f32>(
        sqrt(1.0 - NdotV * NdotV),
        0.0,
        NdotV
    );

    var A: f32 = 0.0;
    var B: f32 = 0.0;

    let N = vec3<f32>(0.0, 0.0, 1.0);

    let SAMPLE_COUNT: u32 = 1024u;
    for (var i: u32 = 0u; i < SAMPLE_COUNT; i = i + 1u) {
        // importance sampling
        let Xi = Hammersley(i, SAMPLE_COUNT);
        let H = ImportanceSampleGGX(Xi, N, roughness);
        let L = normalize(2.0 * dot(V, H) * H - V);

        let NdotL = max(L.z, 0.0);
        let NdotH = max(H.z, 0.0);
        let VdotH = max(dot(V, H), 0.0);

        if (NdotL > 0.0) {
            let G = GeometrySmith_ForIBL(N, V, L, roughness);
            let G_Vis = (G * VdotH) / (NdotH * NdotV);
            let Fc = pow(1.0 - VdotH, 5.0);

            A = A + (1.0 - Fc) * G_Vis;
            B = B + Fc * G_Vis;
        }
    }

    A = A / f32(SAMPLE_COUNT);
    B = B / f32(SAMPLE_COUNT);
    return vec2<f32>(A, B);
}


@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec2<f32> {

    let integratedBRDF = IntegrateBRDF(input.uv.x, input.uv.y);

    return integratedBRDF; 

    // test uv coords
    // return vec2<f32>(input.uv.x, input.uv.y);

}
