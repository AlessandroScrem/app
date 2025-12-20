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

struct Light {
    color: vec3<f32>,
    directional: u32,
    position: vec3<f32>,
    cast_shadow: u32,
    entity_id_low: u32,
    entity_id_high: u32,
    pad2: vec2<u32>,
}

struct Model {
    model: mat4x4<f32>,
    normal_matrix: mat3x3<f32>,
    entity_id_low: u32,
    entity_id_high: u32,
}

struct Material {
    color: vec4<f32>,
    emissive: vec4<f32>,
    roughness: f32,
    metallic: f32,
    normal_scale: f32,
    occlusion_strength: f32,
    use_color_texture: u32,
    use_metal_roughness_texture: u32,
    use_normal_texture: u32,
    use_emissive_texture: u32,
    use_occlusion_texture: u32,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
    @location(3) uv: vec2<f32>,
};

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<uniform> globals: Globals;
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
    out.normal = normalize(model.normal_matrix * vertex.normal);
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

const DebugNone: u32 = 0; 
const DebugBaseColor: u32 = 1; 
const DebugNormal: u32 = 2; 
const DebugMetallic: u32 = 3; 
const DebugRoughness: u32 = 4; 
const DebugOcclusion: u32 = 5; 
const DebugEmissive: u32 = 6; 


@group(1) @binding(0) var tex_sampler: sampler;
@group(1) @binding(1) var albedo_map: texture_2d<f32>;
@group(1) @binding(2) var normal_map: texture_2d<f32>;
// Occlusion (R), Roughness (G), Metallic (B) https://github.com/KhronosGroup/glTF/issues/857
@group(1) @binding(3) var orm_map: texture_2d<f32>; 
@group(1) @binding(4) var emissive_map: texture_2d<f32>; 
@group(1) @binding(5) var occlusion_map: texture_2d<f32>; 
@group(1) @binding(6) var <uniform> material: Material;

@group(3) @binding(0) var<uniform> light: Light;
@group(3) @binding(1) var ibl_sampler: sampler;
@group(3) @binding(2) var irradiance_map: texture_cube<f32>;
@group(3) @binding(3) var prefilter_map: texture_cube<f32>;
@group(3) @binding(4) var brdf_lut_map: texture_2d<f32>;


fn check(value: u32)->bool {return value == 1;}

fn CalculateLight(
    N: vec3<f32>,
    V: vec3<f32>,
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
    frag_pos: vec3<f32>,
) -> vec3<f32> {
    let PI = 3.14159265359;
    // -------------------------------
    // Base reflectivity (F0)
    // -------------------------------
    let F0 = mix(vec3<f32>(0.04, 0.04, 0.04), albedo, metallic);

    var color = vec3<f32>(0.0);
    for (var i: u32 = 0u; i < NUM_LIGHTS; i += 1u) {
        let L =  normalize(light.position - frag_pos);
        let H = normalize(V + L);
        let NdotV = max(dot(N, V), 0.0);
        let NdotL = max(dot(N, L), 0.0);
        let HdotV = max(dot(H, V), 0.0);
        
        var radiance =  vec3<f32>(0.0, 0.0, 0.0);
        if light.directional == 1 {
            radiance = light.color;
        } else {
            let d = length(light.position - frag_pos);
            let attenuation = 1.0 / (d * d);
            radiance = light.color * attenuation;
        };

       // -------------------------------
        // Cook–Torrance BRDF
        // -------------------------------

        // NDF - normal distribution
        let a  = roughness * roughness;
        let a2 = a * a;
        let NdotH = max(dot(N, H), 0.0);
        let NdotH2 = NdotH * NdotH;

        var denomD = (NdotH2 * (a2 - 1.0) + 1.0);
        let D = a2 / (PI * denomD * denomD + 0.00001);

        // Geometry (Smith)
        let k = (roughness + 1.0);
        let k2 = (k * k) / 8.0;
        let G1 = NdotV / (NdotV * (1.0 - k2) + k2 + 0.00001);
        let G2 = NdotL / (NdotL * (1.0 - k2) + k2 + 0.00001);
        let G = G1 * G2;

        // Fresnel
        let F = F0 + (1.0 - F0) * pow(1.0 - HdotV, 5.0);

        // Final specular
        let numerator = D * G * F;
        let denom     = 4.0 * NdotV * NdotL + 0.00001;
        let specular  = numerator / denom;

        // -------------------------------
        // Diffuse term (Lambert)
        // -------------------------------
        let kS = F;
        let kD = (1.0 - kS) * (1.0 - metallic);

        // Lambertian
        let diffuse = (albedo / PI);

        // -------------------------------
        // Final contribution
        // -------------------------------
        color += (kD * diffuse + specular) * radiance * NdotL;  
    }

    return color;
}

fn CalculateAmbient(
    N: vec3<f32>,
    V: vec3<f32>,
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
) -> vec3<f32> {
    if !check(globals.ibl_enable) {
        return vec3<f32> (0.0);
    } 

    let F0 = mix(vec3<f32>(0.04, 0.04, 0.04), albedo, metallic);
    let NdotV = max(dot(N, V), 0.0);
    let R = reflect(-V, N);

    // Fresnel 
    let F =  F0 + (max(vec3(1.0 - roughness), F0) - F0) * pow(clamp(1.0 - NdotV, 0.0, 1.0), 5.0);
    
    let kS = F;
    var kD = (vec3(1.0) - kS) * (1.0 - metallic);

    let irradiance = textureSample(irradiance_map, ibl_sampler, N).rgb;

    let MAX_REFLECTION_LOD: f32 = 4.0;
    let prefiltered_color = textureSampleLevel(prefilter_map, ibl_sampler, R, roughness * MAX_REFLECTION_LOD).rgb;
    let env_brdf = textureSample(brdf_lut_map, ibl_sampler, vec2<f32>(NdotV, roughness)).rg;

    let diffuse = irradiance * albedo * globals.ibl_intensity;
    let specular = prefiltered_color * (F * env_brdf.x + env_brdf.y) * globals.ibl_intensity;

    return (kD * diffuse + specular);
}

struct FSOutput {
    @location(0) color : vec4<f32>,
    @location(1) entity_id : vec2<u32>,
}

@fragment
fn fs_main(input: VertexOutput) -> FSOutput {
    var out: FSOutput;
    var albedo_color = material.color.rgb;
    var metallic = material.metallic;
    var roughness = material.roughness;
    var emissive = vec3<f32>(0.0);
    var ao = 1.0;

    let N = normalize(input.normal);
    let V = normalize(camera.view_pos - input.world_pos);

    if check(material.use_color_texture)  {
        albedo_color *= textureSample(albedo_map, tex_sampler, input.uv).rgb;
    }
    if check(material.use_metal_roughness_texture) {
        roughness *= textureSample(orm_map, tex_sampler, input.uv).g;
        metallic *= textureSample(orm_map, tex_sampler, input.uv).b;
    }

    if check(material.use_occlusion_texture) {
        let occlusion_texture = textureSample(occlusion_map, tex_sampler, input.uv).r;
        ao = 1.0 + material.occlusion_strength * (occlusion_texture - 1.0);
    }

    if check(material.use_emissive_texture) {
        let emissive_texture = textureSample(emissive_map, tex_sampler, input.uv).rgb;
        emissive = emissive_texture * material.emissive.rgb;
    }

    var lo = CalculateLight(N, V, albedo_color, metallic, roughness, input.world_pos);

    var ambient = CalculateAmbient(N, V, albedo_color, metallic, roughness);
    ambient *=  ao; 
    
    var color = lo + ambient + emissive;


    if globals.debug == DebugBaseColor {
        color = albedo_color;
    }
    if globals.debug == DebugNormal {
        color = (N + 1.0) / 2.0;
    }
    if globals.debug == DebugRoughness {
        color = vec3(roughness);
    }
    if globals.debug == DebugMetallic {
        color = vec3(metallic);
    }
    if globals.debug == DebugOcclusion {
        color = vec3(ao);
    }
    if globals.debug == DebugEmissive {
        color = emissive;
    }
    
    out.color = vec4<f32>(color, 1.0);
    out.entity_id =  vec2<u32>(model.entity_id_low, model.entity_id_high);

    return out;
}


    // debug normal
    // color = -N * 0.5 + 0.5;
    // return vec4<f32>(normalize(input.normal) * 0.5 + 0.5, 1.0);

    // debug vettore vista
    // return vec4<f32>((V*0.5+0.5),1.0);
