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

struct Model {
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
}

struct Material {
    color: vec4<f32>,
    roughness: f32,
    metallic: f32,
    roughness_use_texture: u32,
    metallic_use_texture: u32,
    color_use_texture: u32,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
    @location(3) uv: vec2<f32>,
};

@group(0) @binding(0) var<uniform> camera: Camera;
@group(2) @binding(0) var<uniform> model: Model;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
    @location(3) uv: vec2<f32>,
};

@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {

    var out: VertexOutput;
    let world_position = model.model * vec4<f32>(vertex.position, 1.0);

    out.clip_position = camera.proj * camera.view * world_position;
    out.world_pos = world_position.xyz;
    out.normal = normalize((model.normal_matrix * vec4<f32>(vertex.normal, 0.0)).xyz);
    out.uv =  vertex.uv;
    out.color = vertex.color;

    return out;
}

/// Fragment shader
///
const NUM_LIGHTS: u32 = 1u;
const LIGHT_TARGET: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);
const AMBIENT_COLOR: vec3<f32> = vec3<f32>(0.2, 0.2, 0.2);
const MATERIAL_SHININESS: f32 = 4.0;
const MATERIAL_SPECULAR: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0);


@group(1) @binding(0) var tex_sampler: sampler;
@group(1) @binding(1) var albedo_map: texture_2d<f32>;
@group(1) @binding(2) var normal_map: texture_2d<f32>;
@group(1) @binding(3) var roughness_map: texture_2d<f32>;
@group(1) @binding(4) var <uniform> material: Material;

@group(3) @binding(0) var<uniform> light: Light;
@group(3) @binding(1) var ibl_sampler: sampler;
@group(3) @binding(2) var irradiance_map: texture_cube<f32>;
@group(3) @binding(3) var prefilter_map: texture_cube<f32>;
@group(3) @binding(4) var brdf_lut_map: texture_2d<f32>;


fn DistributionGGX(N: vec3<f32>, H: vec3<f32>, roughness: f32) ->f32 {
  var PI = 3.14159265359;
  var a = roughness * roughness;
  var a2 = a * a;
  let NdotH = max(dot(N, H), 0.0);
  let NdotH2 = NdotH * NdotH;

  let nom = a2;
  var denom = (NdotH2 * (a2 - 1.0) + 1.0);
  denom = PI * denom * denom;

  return nom / denom;
}

fn GeometrySchlickGGX(NdotV: f32, roughness: f32, k: f32) ->f32 {
  let nom = NdotV;
  let denom = NdotV * (1.0 - k) + k;
  return nom / denom;
}

fn GeometrySmith_kdirect(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) ->f32{
  let a = (roughness + 1.0);
  let k = (a * a) / 8.0;

  let NdotV = max(dot(N, V), 0.0);
  let NdotL = max(dot(N, L), 0.0);
  let ggx2 = GeometrySchlickGGX(NdotV, roughness, k);
  let ggx1 = GeometrySchlickGGX(NdotL, roughness, k);

  return ggx1 * ggx2;
}

fn fresnelSchlick(cosTheta: f32, F0: vec3<f32>) ->vec3<f32> {
  return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

fn fresnelSchlickRoughness(cosTheta: f32, F0: vec3<f32>, roughness: f32) ->vec3<f32> {
  return F0 + (max(vec3(1.0 - roughness), F0) - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

fn CalculateLight(
    N: vec3<f32>,
    V: vec3<f32>,
    F0: vec3<f32>,
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
    frag_pos: vec3<f32>,
    num_lights: u32,
) -> vec3<f32> {
    var PI = 3.14159265359;
    var color = vec3<f32>(0.0, 0.0, 0.0);

    for (var i: u32 = 0u; i < num_lights; i = i + 1u) {
        let L =  normalize(light.position - frag_pos);
        let H = normalize(V + L);
        
        var attenuation = 1.0;
        if (light.directional == 0u) {
            let distance = length(light.position - frag_pos);
            attenuation = 1.0 / (distance * distance);
        };

        let radiance = light.color * attenuation;

        let NDF = DistributionGGX(N, H, roughness);
        let G   = GeometrySmith_kdirect(N, V, L, roughness);
        let F   = fresnelSchlick(max(dot(H, V), 0.0), F0);

         // --------- Cook-Torrance specular BRDF ----------
        let numerator = NDF * G * F;
        let denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001; // + 0.0001 to prevent divide by zero
        let specular = numerator / denominator;

        // kS represents the energy of light that gets reflected, is equal to Fresnel
        let kS = F;
        var kD = vec3<f32>(1.0, 1.0, 1.0) - kS;
        kD = kD * (1.0 - metallic);

        let diffuse:vec3<f32> = kD * albedo / PI;

        let NdotL = max(dot(N, L), 0.0);

        color += (diffuse + specular) * radiance * NdotL;
    }

    return color;
}

fn CalculateAmbient(
    N: vec3<f32>,
    V: vec3<f32>,
    R: vec3<f32>,
    F0: vec3<f32>,
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
) -> vec3<f32> {
    let F = fresnelSchlickRoughness(max(dot(N, V), 0.0), F0, roughness);
    let kS = F;
    var kD = vec3<f32>(1.0, 1.0, 1.0) - kS;
    kD = kD * (1.0 - metallic);

    let MAX_REFLECTION_LOD: f32 = 4.0;
    let prefiltered_color = textureSampleLevel(prefilter_map, ibl_sampler, R, roughness * MAX_REFLECTION_LOD).rgb;
    let NdotV: f32 = max(dot(N, V), 0.0);
    let env_brdf = textureSample(brdf_lut_map, ibl_sampler, vec2<f32>(NdotV, roughness)).rg;

    let irradiance = textureSample(irradiance_map, ibl_sampler, N).rgb;
    let diffuse = irradiance * albedo;
    let specular = prefiltered_color * (F * env_brdf.x + env_brdf.y);

    return (kD * diffuse + specular);
    // debug
    // return vec3<f32>(env_brdf.y);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    var albedo_color = material.color.rgb;
    var metallic = material.metallic;
    var roughness = material.roughness;

    let N = normalize(input.normal);
    let V = normalize(camera.view_pos - input.world_pos);
    let R = reflect(-V, N);

    if (material.color_use_texture == 1u) {
        albedo_color = textureSample(albedo_map, tex_sampler, input.uv).rgb;
    }

    if (material.metallic_use_texture == 1u) {
        metallic = textureSample(roughness_map, tex_sampler, input.uv).r;
    }

    if (material.roughness_use_texture == 1u) {
        roughness = textureSample(roughness_map, tex_sampler, input.uv).g;
    }

    let F0 = mix(vec3<f32>(0.04, 0.04, 0.04), albedo_color, metallic);

    var color = vec3<f32>(0.0, 0.0, 0.0);
    color += CalculateLight(N, V, F0, albedo_color, metallic, roughness, input.world_pos, NUM_LIGHTS);
    color += CalculateAmbient(N, V, R, F0, albedo_color, metallic, roughness);

    // debug normal
    // color = -N * 0.5 + 0.5;
    // return vec4<f32>(normalize(input.normal) * 0.5 + 0.5, 1.0);

    // debug vettore vista
    // return vec4<f32>((V*0.5+0.5),1.0);

    return vec4<f32>(color, 1.0);
}