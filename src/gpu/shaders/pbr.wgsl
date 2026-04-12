const MAX_LIGHTS             : u32 = 64;
const MAX_REFLECTION_LOD     : f32 = 7.0; // max mips on "prefilter_map" (texture.mip_level_count() -1)
const MAX_SCENE_LOD          : f32 = 7.0; // max mips on "scene_color" (texture.mip_level_count() -1) 

const COLOR_TEXTURE          : u32 = 1u << 0u;
const NORMAL_TEXTURE         : u32 = 1u << 1u;
const METAL_ROUGHNESS_TEXTURE: u32 = 1u << 2u;
const EMISSIVE_TEXTURE       : u32 = 1u << 3u;
const OCCLUSION_TEXTURE      : u32 = 1u << 4u;
const TRANSMISSION_TEXTURE   : u32 = 1u << 5u;

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

    env_rotation: f32,
};

struct Light {
    color: vec3<f32>,
    directional: u32,

    position: vec3<f32>,
    cast_shadow: u32,
    
    entity_id_low: u32,
    entity_id_high: u32,
    enabled: u32,
}

struct Lights {
    lights: array<Light, MAX_LIGHTS>,
    
    count: u32,
    enabled: u32, 
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

    roughness_factor: f32,
    metallic_factor: f32,
    normal_scale: f32,
    occlusion_strength: f32,

    texture_flags: u32,
    alpha_mode: u32,
    alpha_cutoff: f32,
    transmission_factor: f32,

    is_trasmissive: u32,
}

struct VertexInput {
    @location(0) position : vec3<f32>,
    @location(1) normal   : vec3<f32>,
    @location(2) tangent  : vec4<f32>, // xyz = T, w = sign
    @location(3) uv       : vec2<f32>,
};

// PerFrame
@group(0) @binding(0) var<uniform> camera  : Camera;
@group(0) @binding(1) var<uniform> globals : Globals;
@group(0) @binding(2) var<uniform> lights  : Lights;
@group(2) @binding(0) var<uniform> model   : Model;

struct VertexOutput {
    @builtin(position) clip_position : vec4<f32>,
    @location(0) world_pos           : vec3<f32>,
    @location(1) normal              : vec3<f32>,
    @location(2) tangent             : vec4<f32>, // xyz = T, w = sign
    @location(3) uv                  : vec2<f32>,
    @location(4) ndc_xy              : vec2<f32>, 
};

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {

    var out: VertexOutput;
    let world_position = model.model * vec4<f32>(in.position, 1.0);

    let clip = camera.proj * camera.view * world_position;

    out.clip_position = clip;
    out.world_pos = world_position.xyz;

    out.tangent = vec4(normalize(model.normal_matrix * in.tangent.xyz), in.tangent.w);
    out.normal  = normalize(model.normal_matrix * in.normal);
    out.uv =  in.uv;

    // salva NDC
    out.ndc_xy = clip.xy / clip.w; // [-1, 1]

    return out;
}


/// Fragment shader
///

const True                   : u32 = 1;
const False                  : u32 = 0;

const AlphaMask              : u32 = 1; 

const DebugNone              : u32 = 0; 
const DebugBaseColor         : u32 = 1; 
const DebugNormalTexture     : u32 = 2; 
const DebugGeometryNormal    : u32 = 3; 
const DebugGeometryTangent   : u32 = 4; 
const DebugGeometryBitangent : u32 = 5; 
const DebugGeometryTangentW  : u32 = 6; 
const DebugMetallic          : u32 = 7; 
const DebugRoughness         : u32 = 8; 
const DebugEmissive          : u32 = 9; 
const DebugOcclusion         : u32 = 10; 
const DebugTransmission      : u32 = 11; 

// Material
@group(1) @binding(0) var <uniform> material: Material;
@group(1) @binding(1) var tex_sampler: sampler;
@group(1) @binding(2) var albedo_map: texture_2d<f32>;
@group(1) @binding(3) var normal_map: texture_2d<f32>;
@group(1) @binding(4) var orm_map: texture_2d<f32>;         // Occlusion (R), Roughness (G), Metallic (B) https://github.com/KhronosGroup/glTF/issues/857
@group(1) @binding(5) var emissive_map: texture_2d<f32>; 
@group(1) @binding(6) var occlusion_map: texture_2d<f32>; 
@group(1) @binding(7) var transmission_map: texture_2d<f32>; 

// Ibl 
@group(3) @binding(0) var ibl_sampler: sampler;
@group(3) @binding(1) var irradiance_map: texture_cube<f32>;
@group(3) @binding(2) var prefilter_map: texture_cube<f32>; // miplevels = 5
@group(3) @binding(3) var brdf_lut_map: texture_2d<f32>;
@group(3) @binding(4) var scene_sampler: sampler;           // transmission input scene sampler
@group(3) @binding(5) var scene_color: texture_2d<f32>;  

// calculate mat env_rotation from angle(rad)
fn env_rotY() -> mat3x3<f32> {
    let angle = globals.env_rotation;
    let c = cos(angle);
    let s = sin(angle);

    return mat3x3<f32>(
        vec3<f32>( c, 0.0, -s),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>( s, 0.0,  c)
    );
}

struct LightResult {
    diffuse: vec3<f32>,
    specular: vec3<f32>,
};

fn CalculateLight(
    N: vec3<f32>,
    V: vec3<f32>,
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
    frag_pos: vec3<f32>,
) -> LightResult {
    let PI = 3.14159265359;

    var result: LightResult;
    result.diffuse = vec3<f32>(0.0);
    result.specular = vec3<f32>(0.0);

    if lights.enabled == False {
        return result;
    }

    for (var i: u32 = 0u; i < lights.count; i += 1u) {
        let light = lights.lights[i];
        if light.enabled == False {
            continue;
        }

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

        // -------------------
        // NDF - normal distribution
        // -------------------
        let a  = roughness * roughness;
        let a2 = a * a;
        let NdotH = max(dot(N, H), 0.0);
        let NdotH2 = NdotH * NdotH;

        var denomD = (NdotH2 * (a2 - 1.0) + 1.0);
        let D = a2 / (PI * denomD * denomD + 0.00001);

        // -------------------
        // Geometry (Smith)
        // -------------------
        let k = (roughness + 1.0);
        let k2 = (k * k) / 8.0;
        let G1 = NdotV / (NdotV * (1.0 - k2) + k2 + 0.00001);
        let G2 = NdotL / (NdotL * (1.0 - k2) + k2 + 0.00001);
        let G = G1 * G2;

        // Base reflectivity (F0)
        let F0 = mix(vec3<f32>(0.04, 0.04, 0.04), albedo, metallic);
        
        // -------------------
        // F (Fresnel)
        // -------------------
        let F = F0 + (1.0 - F0) * pow(1.0 - HdotV, 5.0);

        // -------------------
        // Specular
        // -------------------
        let numerator = D * G * F;
        let denom     = 4.0 * NdotV * NdotL + 0.00001;
        let specular  = numerator / denom;

        // -------------------------------
        // Diffuse (Lambert)
        // -------------------------------
        let kS = F;
        let kD = (1.0 - kS) * (1.0 - metallic);
        let diffuse = (albedo / PI);

        // -------------------------------
        // Final contribution
        // -------------------------------
        result.diffuse  += kD * diffuse * radiance * NdotL;  
        result.specular += specular * radiance * NdotL;  
    }

    return result;
}

fn CalculateAmbient(
    N: vec3<f32>,
    V: vec3<f32>,
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
) -> LightResult {
    var result: LightResult;
    result.diffuse = vec3<f32>(0.0);
    result.specular = vec3<f32>(0.0);
    
    if globals.ibl_enable == False {
        return result;
    } 

    let env_rotation = env_rotY();

    // Base reflectivity (F0)
    let F0 = mix(vec3<f32>(0.04, 0.04, 0.04), albedo, metallic);
    let NdotV = max(dot(N, V), 0.0);
    let R = reflect(-V, N);

    // Fresnel 
    let F =  F0 + (max(vec3(1.0 - roughness), F0) - F0) * pow(clamp(1.0 - NdotV, 0.0, 1.0), 5.0);
    
    let kS = F;
    var kD = (vec3(1.0) - kS) * (1.0 - metallic);

    let irradiance = textureSample(irradiance_map, ibl_sampler, env_rotation * N).rgb;

    var lod = roughness * MAX_REFLECTION_LOD;

    let prefiltered = textureSampleLevel(prefilter_map, ibl_sampler, env_rotation * R, lod).rgb;
    let brdf = textureSample(brdf_lut_map, ibl_sampler, vec2<f32>(NdotV, roughness)).rg;

    result.diffuse = irradiance * albedo * kD *  globals.ibl_intensity;
    result.specular = prefiltered * (F * brdf.x + brdf.y) * globals.ibl_intensity;

    return result;
}

fn has_flag(flags: u32, bit: u32) -> bool {
    return (flags & bit) != 0u;
}


fn get_color(uv: vec2<f32>) ->vec3<f32> {
    var albedo_color = material.color.rgb;
    if has_flag(material.texture_flags, COLOR_TEXTURE)  {
        albedo_color *= textureSample(albedo_map, tex_sampler, uv).rgb;
    }
    return albedo_color;
}


fn get_alpha(uv: vec2<f32>) ->f32 {
    return textureSample(albedo_map, tex_sampler, uv).a;
}

fn get_metallic(uv: vec2<f32>) ->f32 {
    var metallic = material.metallic_factor;
    if has_flag(material.texture_flags, METAL_ROUGHNESS_TEXTURE) {
        metallic *= textureSample(orm_map, tex_sampler, uv).b;
        metallic = select(0.0, metallic, metallic > 0.06); //select(false_value, true_value, condition)
    }
    return clamp(metallic, 0.0, 1.0);
}

fn get_roughness(uv: vec2<f32>) ->f32 {
    var roughness = material.roughness_factor;
    if has_flag(material.texture_flags, METAL_ROUGHNESS_TEXTURE) {
        roughness *= textureSample(orm_map, tex_sampler, uv).g;
    }
    return clamp(roughness, 0.08, 1.0);
}

fn get_occlusion(uv: vec2<f32>) ->f32 {
    var ao:f32 = 1.0;
    if has_flag(material.texture_flags, OCCLUSION_TEXTURE) {
        let occlusion_texture = textureSample(occlusion_map, tex_sampler, uv).r;
        ao = 1.0 + material.occlusion_strength * (occlusion_texture - 1.0);
    }
    return ao;
}

fn get_emissive(uv: vec2<f32>) ->vec3<f32> {
    var emissive = vec3<f32>(0.0);
    if has_flag(material.texture_flags, EMISSIVE_TEXTURE) {
        let emissive_texture = textureSample(emissive_map, tex_sampler, uv).rgb;
        emissive = emissive_texture * material.emissive.rgb;
    }
    return emissive;
}

fn get_normal_texture(uv: vec2<f32>) ->vec3<f32> {
    var normal_ts = vec3<f32>(0.0);
    if has_flag(material.texture_flags, NORMAL_TEXTURE) {
        normal_ts = textureSample(normal_map, tex_sampler, uv).rgb;
        normal_ts =  normal_ts * 2.0 - 1.0;            // map to [-1, 1.0]

        normal_ts = vec3<f32>( 
            normal_ts.xy * material.normal_scale, 
            normal_ts.z
        );

        normal_ts = normalize(normal_ts);
    }
    return normal_ts;
}

fn get_transmission(uv: vec2<f32>) ->f32 {
    var transmission = material.transmission_factor;
    if has_flag(material.texture_flags, TRANSMISSION_TEXTURE) {
        transmission *= textureSample(transmission_map, tex_sampler, uv).r;
    }
    return transmission;
}

fn inverse_srgb(c: vec3<f32>) -> vec3<f32> {
    var result: vec3<f32>;
    for (var i: u32 = 0u; i < 3u; i = i + 1u) {
        if (c[i] <= 0.04045) {
            result[i] = c[i] / 12.92;
        } else {
            result[i] = pow((c[i] + 0.055) / 1.055, 2.4);
        }
    }
    return result;
}

// transmission no refraction
fn computeTransmission(
    N:            vec3<f32>,   
    V:            vec3<f32>,
    albedo_color: vec3<f32>,   
    transmission: f32,
    metallic:     f32,
    roughness:    f32,
    ndc_xy:       vec2<f32>,
) -> vec3<f32> {

    // Fresnel (F0)
    let NdotV = max(dot(N, V), 0.0);
    let F0 = mix(vec3<f32>(0.04), albedo_color, metallic);
    let F = F0 + (1.0 - F0) * pow(1.0 - NdotV, 5.0);

    // Normalizza NDC in UV [0,1]
    var uv = ndc_xy * 0.5 + vec2(0.5);
    uv.y = 1.0 - uv.y; // inverti Y se necessario

    // Campiona con mip-level
    let lod = roughness * MAX_SCENE_LOD;
    let color = textureSampleLevel(scene_color, scene_sampler, uv, lod).rgb;
    let transmitted = color * transmission;

    let transmitted_tinted = mix(transmitted, transmitted * albedo_color, 1.0);

    return transmitted_tinted * (1.0 - F);
}

struct FSOutput {
    @location(0) color : vec4<f32>,
    @location(1) entity_id : vec2<u32>,
}

@fragment
fn fs_main(
    in: VertexOutput, 
    @builtin(front_facing) is_front_facing: bool
) -> FSOutput {
    var out: FSOutput;

    let alpha = get_alpha(in.uv);
    if material.alpha_mode == AlphaMask  && alpha < material.alpha_cutoff && globals.debug == False  {
        discard;
    }

    let albedo_color = get_color(in.uv);
    let normal_texture = get_normal_texture(in.uv);
    let metallic = get_metallic(in.uv);
    let roughness = get_roughness(in.uv);
    let emissive = get_emissive(in.uv);
    let ao = get_occlusion(in.uv);
    let transmission =  get_transmission(in.uv);


    let N = normalize(in.normal);
    let T = normalize(in.tangent.xyz);
    let B = in.tangent.w * normalize(cross(N, T));

    // Convert normal to world space
    var Nws = N;
    if has_flag(material.texture_flags, NORMAL_TEXTURE) {
        let TBN = mat3x3<f32>(T, B, N);
        Nws = normalize(TBN * normal_texture);
    }

    // Check frontfacing: (fix alpha mask surfaces, fix faces reversed from camera view)
    Nws = select(-Nws, Nws, is_front_facing); //select(false_value, true_value, condition)

    let V = normalize(camera.view_pos - in.world_pos);

    let light_res = CalculateLight(Nws, V, albedo_color, metallic, roughness, in.world_pos);
    let ambient_res = CalculateAmbient(Nws, V, albedo_color, metallic, roughness);

    let diffuse = light_res.diffuse + ambient_res.diffuse * ao;
    let specular = light_res.specular + ambient_res.specular * ao;

    var color = vec3<f32>();
    if material.is_trasmissive == True {
        var transmission_color = computeTransmission(Nws, V, albedo_color, transmission, metallic, roughness, in.ndc_xy);

        color = specular + 
            transmission_color * transmission +
            diffuse * (1.0 - transmission);

    } else {
        color = diffuse + specular + emissive; 
    }

    switch globals.debug {
        case DebugBaseColor         : { color = albedo_color; }
        case DebugNormalTexture     : { color = inverse_srgb((normal_texture + 1.0) * 0.5);}
        case DebugGeometryNormal    : { color = inverse_srgb((N + 1.0) * 0.5 );}
        case DebugGeometryTangent   : { color = inverse_srgb((T + 1.0) * 0.5 );}
        case DebugGeometryBitangent : { color = inverse_srgb((B + 1.0) * 0.5 );}
        case DebugGeometryTangentW  : { color = inverse_srgb(vec3(in.tangent.w + 1.0) * 0.5);}
        case DebugMetallic          : { color = inverse_srgb(vec3(metallic));}
        case DebugRoughness         : { color = inverse_srgb(vec3(roughness));}
        case DebugEmissive          : { color = emissive;}
        case DebugOcclusion         : { color = inverse_srgb(vec3(ao));}
        case DebugTransmission      : { color = vec3(transmission);}
        default: {;} 
    }
    
    // attachement 0:
    out.color = vec4<f32>(color, 1.0);
    // attachement 1:
    out.entity_id =  vec2<u32>(model.entity_id_low, model.entity_id_high);

    return out;
}


    // debug normal
    // color = -N * 0.5 + 0.5;
    // return vec4<f32>(normalize(in.normal) * 0.5 + 0.5, 1.0);

    // debug vettore vista
    // return vec4<f32>((V*0.5+0.5),1.0);
