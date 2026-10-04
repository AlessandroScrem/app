/// Vertex shader

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};


@vertex
fn vs_main(@builtin(vertex_index) vi: u32,
) -> VertexOutput {
    var out: VertexOutput;
    // Generate a triangle that covers the whole screen
    out.uv = vec2<f32>(
        f32((vi << 1u) & 2u),
        f32(vi & 2u),
    );
    out.clip_position = vec4<f32>(out.uv * 2.0 - 1.0, 0.0, 1.0);
    // We need to invert the y coordinate so the image
    // is not upside down
    out.uv.y = 1.0 - out.uv.y;
    return out;
}

struct Globals {
    ibl_enable: u32,
    skybox_enable: u32,
    exposure: f32,
    ibl_intensity: f32,
    
    selected_entity_id_low: u32,
    selected_entity_id_high: u32,
    tonemap_filter: u32,
    debug: u32,
    
    env_rotation: f32,
};

/// Fragment shader
///

// preso da basic_glfw cpp
fn aces(x: vec3<f32> )->vec3<f32> {
  let a:f32 = 2.51;
  let b:f32 = 0.03;
  let c:f32 = 2.43;
  let d:f32 = 0.59;
  let e:f32 = 0.14;

  let val: vec3<f32> = (x * (a * x + b)) / (x * (c * x + d) + e);
  return clamp(val , vec3(0.0), vec3(1.0));
}


////////////////////////////////////////////////////////////////////////////////
// Filmic Tonemapping Operators http://filmicworlds.com/blog/filmic-tonemapping-operators/
fn filmic(x: vec3<f32>)-> vec3<f32> {
  let X = max(vec3(0.0), x - 0.004);
  let result = (X * (6.2 * X + 0.5)) / (X * (6.2 * X + 1.7) + 0.06);
  return pow(result, vec3(2.2));
}

////////////////////////////////////////////////////////////////////////////////
// Lottes 2016, "Advanced Techniques and Optimization of HDR Color Pipelines"
fn lottes(x: vec3<f32>) ->vec3<f32> {
  let  a = vec3(1.6);
  let  d = vec3(0.977);
  let  hdrMax = vec3(8.0);
  let  midIn = vec3(0.18);
  let  midOut = vec3(0.267);

  let b:vec3<f32> =
      (-pow(midIn, a) + pow(hdrMax, a) * midOut) /
      ((pow(hdrMax, a * d) - pow(midIn, a * d)) * midOut);
  let c:vec3<f32> =
      (pow(hdrMax, a * d) * pow(midIn, a) - pow(hdrMax, a) * pow(midIn, a * d) * midOut) /
      ((pow(hdrMax, a * d) - pow(midIn, a * d)) * midOut);

  return pow(x, a) / (pow(x, a * d) * b + c);
}

////////////////////////////////////////////////////////////////////////////////
// Reinhard
fn reinhard(x: vec3<f32>) -> vec3<f32> {
  return x / (1.0 + x);
}

////////////////////////////////////////////////////////////////////////////////
// Reinhard II (variant)
fn reinhard2(x: vec3<f32>) ->vec3<f32> {
  let L_white = 4.0;

  return (x * (1.0 + x / (L_white * L_white))) / (1.0 + x);
}

////////////////////////////////////////////////////////////////////////////////
// Uchimura 2017, "HDR theory and practice"
// Math: https://www.desmos.com/calculator/gslcdxvipg
// Source: https://www.slideshare.net/nikuque/hdr-theory-and-practicce-jp
fn uchimura(x: vec3<f32>) -> vec3<f32> {
    let P: f32 = 1.0;   // max display brightness
    let a: f32 = 1.0;   // contrast
    let m: f32 = 0.22;  // linear section start
    let l: f32 = 0.4;   // linear section length
    let c: f32 = 1.33;  // black
    let b: f32 = 0.0;   // pedestal

    let l0: f32 = ((P - m) * l) / a;
    let L0: f32 = m - m / a;
    let L1: f32 = m + (1.0 - m) / a;
    let S0: f32 = m + l0;
    let S1: f32 = m + a * l0;
    let C2: f32 = (a * P) / (P - S1);
    let CP: f32 = -C2 / P;

    let w0: vec3<f32> = vec3<f32>(1.0) - smoothstep(vec3<f32>(0.0), vec3<f32>(m), x);
    let w2: vec3<f32> = step(vec3<f32>(m + l0), x);
    let w1: vec3<f32> = vec3<f32>(1.0) - w0 - w2;

    let T: vec3<f32> = vec3<f32>(m) * pow(x / vec3<f32>(m), vec3<f32>(c)) + vec3<f32>(b);
    let S: vec3<f32> = vec3<f32>(P) - vec3<f32>(P - S1) * exp(vec3<f32>(CP) * (x - vec3<f32>(S0)));
    let L: vec3<f32> = vec3<f32>(m) + vec3<f32>(a) * (x - vec3<f32>(m));

    return T * w0 + L * w1 + S * w2;
}

////////////////////////////////////////////////////////////////////////////////
// Uncharted 2
fn uncharted2(x: vec3<f32>) ->vec3<f32>{
  let  A: f32 = 0.15;
  let  B: f32 = 0.50;
  let  C: f32 = 0.10;
  let  D: f32 = 0.20;
  let  E: f32 = 0.02;
  let  F: f32 = 0.30;
  let  W: f32 = 11.2;
  return ((x * (A * x + C * B) + D * E) / (x * (A * x + B) + D * F)) - E / F;
}

////////////////////////////////////////////////////////////////////////////////
// Expose Tonemapping 
// Ref: https://learnopengl.com/Advanced-Lighting/HDR
fn exponential(color: vec3<f32>) ->vec3<f32>{
  return  vec3(1.0) - exp(-color /* * exposure */); 
}

fn toneMap_KhronosPbrNeutral(in_color: vec3<f32> ) ->vec3<f32>
{
    var color:vec3<f32> = in_color;
    let startCompression :f32 = 0.8 - 0.04;
    let desaturation     :f32 = 0.15;

    let x      :f32 = min(color.r, min(color.g, color.b));
    let offset :f32 = select(0.04, x - 6.25 * x * x, x < 0.08); //select(false_value, true_value, condition)
    color -= offset;

    let peak :f32 = max(color.r, max(color.g, color.b));
    if (peak < startCompression) { return color; }

    let d       :f32 = 1. - startCompression;
    let newPeak :f32 = 1. - d * d / (peak + d - startCompression);
    color *= newPeak / peak;

    let g :f32 = 1. - 1. / (desaturation * (peak - newPeak) + 1.);
    return mix(color, newPeak * vec3(1, 1, 1), g);
}


// let tonemap_filters = ["ACES", "Filmic", "Lottes", "Reinhard", "Reinhard2", "Uchimura", "Uncharted2", "Exponential"];
fn tonemap(hdr: vec3<f32>) ->vec3<f32> {
    switch globals.tonemap_filter 
    {
        case 0u: { return toneMap_KhronosPbrNeutral(hdr);}
        case 1u: { return aces(hdr);}
        case 2u: { return filmic(hdr);}
        case 3u: { return lottes(hdr);}
        case 4u: { return reinhard(hdr);}
        case 5u: { return reinhard2(hdr);}
        case 6u: { return uchimura(hdr);}
        case 7u: { return uncharted2(hdr);}
        case 8u: { return exponential(hdr);}
        default: { return aces(hdr); }
    }
}


@group(0) @binding(0) var hdr_sampler: sampler;
@group(0) @binding(1) var hdr_image: texture_2d<f32>;
@group(1) @binding(1) var<uniform> globals: Globals;

@fragment
fn fs_main(vs: VertexOutput) -> @location(0) vec4<f32> {
    var hdr = textureSample(hdr_image, hdr_sampler, vs.uv);
    
    // Se siamo in debug, bypass tonemap/gamma
    if globals.debug != 0u {
        return vec4(hdr.rgb, hdr.a);
    }

    hdr = hdr * globals.exposure;
    
    let sdr = tonemap(hdr.rgb);
    return vec4(sdr, hdr.a);
}