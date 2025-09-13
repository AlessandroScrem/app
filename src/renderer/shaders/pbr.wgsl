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

fn CalculateLight(
    N: vec3<f32>,
    V: vec3<f32>,
    F0: vec3<f32>,
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
    num_lights: u32,
) -> vec3<f32> {
    var color = vec3<f32>(0.0, 0.0, 0.0);

    for (var i: u32 = 0u; i < num_lights; i = i + 1u) {
        var L: vec3<f32>;
        if (light.directional == 1u) {
            L = normalize(-light.position);
        } else {
            L = normalize(light.position - camera.view_pos);
        }
        let H = normalize(V + L);
        let distance = length(light.position - camera.view_pos);
        let attenuation = 1.0 / (distance * distance);
        let radiance = light.color * attenuation;

        // Cook-Torrance BRDF
        let NDF = pow(max(dot(N, H), 0.0), (roughness * roughness) * MATERIAL_SHININESS);
        let G = min(1.0, min((2.0 * dot(N, H) * dot(N, V)) / dot(V, H), (2.0 * dot(N, H) * dot(N, L)) / dot(V, H)));
        let F = F0 + (1.0 - F0) * pow(1.0 - max(dot(H, V), 0.0), 5.0);

        let numerator = NDF * G * F;
        let denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.001;
        let specular = numerator / denominator;

        let kS = F;
        var kD = vec3<f32>(1.0, 1.0, 1.0) - kS;
        kD = kD * (1.0 - metallic);

        let NdotL = max(dot(N, L), 0.0);
        color += (kD * albedo / vec3<f32>(3.14159265359, 3.14159265359, 3.14159265359) + specular) * radiance * NdotL;
    }

    return color;
}

fn _CalculateAmbient(
    N: vec3<f32>,
    V: vec3<f32>,
    R: vec3<f32>,
    F0: vec3<f32>,
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
) -> vec3<f32> {
    let kS = F0 + (1.0 - F0) * pow(1.0 - max(dot(N, V), 0.0), 5.0);
    var kD = vec3<f32>(1.0, 1.0, 1.0) - kS;
    kD = kD * (1.0 - metallic);

    let irradiance = AMBIENT_COLOR;
    let diffuse = irradiance * albedo;

    // Approximate specular IBL
    let MAX_REFLECTION_LOD: f32 = 4.0;
    let prefiltered_color = AMBIENT_COLOR; // textureSampleLod(prefilter_map, tex_sampler, R.xy, roughness * MAX_REFLECTION_LOD).rgb;
    let env_brdf = vec2<f32>(0.04, 0.5); // textureSample(brdf_lut, tex_sampler, vec2<f32>(max(dot(N, V), 0.0), roughness)).rg;
    let specular = prefiltered_color * (kS * env_brdf.x + env_brdf.y);

    return (kD * diffuse + specular);
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
    let kS = F0 + (1.0 - F0) * pow(1.0 - max(dot(N, V), 0.0), 5.0);
    var kD = vec3<f32>(1.0, 1.0, 1.0) - kS;
    kD = kD * (1.0 - metallic);

    let MAX_REFLECTION_LOD: f32 = 4.0;
    let prefiltered_color = textureSampleLevel(prefilter_map, ibl_sampler, R, roughness * MAX_REFLECTION_LOD).rgb;
    let NdotV: f32 = max(dot(N, V), 0.0);
    let env_brdf = textureSample(brdf_lut_map, ibl_sampler, vec2<f32>(NdotV, roughness)).rg;

    let irradiance = textureSample(irradiance_map, ibl_sampler, N).rgb;
    let diffuse = irradiance * albedo;
    let specular = prefiltered_color * (kS * env_brdf.x + env_brdf.y);

    return (kD * diffuse + specular);
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
    color += CalculateLight(N, V, F0, albedo_color, metallic, roughness, NUM_LIGHTS);
    color += CalculateAmbient(N, V, R, F0, albedo_color, metallic, roughness);

    return vec4<f32>(color, 1.0);
}