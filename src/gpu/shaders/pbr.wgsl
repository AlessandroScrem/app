const MAX_LIGHTS             : u32 = 64;
const MAX_REFLECTION_LOD     : f32 = 7.0; // max mips on "prefilter_map" (texture.mip_level_count() -1)
const MAX_SCENE_LOD          : f32 = 7.0; // max mips on "scene_color" (texture.mip_level_count() -1) 
const TEX_SLOT_COUNT         : u32 = 7;

const COLOR_TEXTURE          : u32 = 0u;
const NORMAL_TEXTURE         : u32 = 1u;
const METAL_ROUGHNESS_TEXTURE: u32 = 2u;
const EMISSIVE_TEXTURE       : u32 = 3u;
const OCCLUSION_TEXTURE      : u32 = 4u;
const TRANSMISSION_TEXTURE   : u32 = 5u;
const VOLUME_TEXTURE         : u32 = 6u;

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
    is_volume: u32,
    thickness_factor: f32,
    attenuation_distance: f32,

    attenuation_color: vec3<f32>,
    ior: f32,

    sheen_color_factor: vec3<f32>,
    sheen_roughness_factor: f32,

    texture_transform: array<mat3x3<f32>, TEX_SLOT_COUNT>,

    coord_flags: u32, 
    is_sheen: u32,
}

struct VertexInput {
    @location(0) position : vec3<f32>,
    @location(1) normal   : vec3<f32>,
    @location(2) tangent  : vec4<f32>, // xyz = T, w = sign
    @location(3) uv       : vec4<f32>, // xy = uv0, zw = uv1
};

struct InstanceInput {
    @location(5) model0 : vec4<f32>,
    @location(6) model1 : vec4<f32>,
    @location(7) model2 : vec4<f32>,
    @location(8) model3 : vec4<f32>,
    
    @location(9) normal0 : vec3<f32>,
    @location(10) normal1 : vec3<f32>,
    @location(11) normal2 : vec3<f32>,

    @location(12) entity_id_low: u32,
    @location(13) entity_id_high: u32,
}

fn mat4_from_instance(i: InstanceInput) -> mat4x4<f32> {
    return mat4x4<f32>(
        i.model0,
        i.model1,
        i.model2,
        i.model3
    );
}

fn compute_scale(model: mat4x4<f32>) -> f32 {
    let sx = length(model[0].xyz);
    let sy = length(model[1].xyz);
    let sz = length(model[2].xyz);
    return (sx + sy + sz) * 0.33;
}

fn mat3_from_instance(i: InstanceInput) -> mat3x3<f32> {
    return mat3x3<f32>(
        i.normal0,
        i.normal1,
        i.normal2,
    );
}

// PerFrame
@group(0) @binding(0) var<uniform> camera  : Camera;
@group(0) @binding(1) var<uniform> globals : Globals;
@group(0) @binding(2) var<uniform> lights  : Lights;

struct VertexOutput {
    @builtin(position) clip_position : vec4<f32>,
    @location(0) world_pos           : vec3<f32>,
    @location(1) normal              : vec3<f32>,
    @location(2) tangent             : vec4<f32>, // xyz = T, w = sign
    @location(3) uv                  : vec4<f32>,
    @location(4) ndc_xy              : vec2<f32>, 
    @location(5) entity_id           : vec2<u32>, 
    @location(6) world_scale         : f32,  //  world scale
};

@vertex
fn vs_main(
    in: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {

    var out: VertexOutput;
    let model = mat4_from_instance(instance);
    let normal_matrix = mat3_from_instance(instance);

    let world_position = model * vec4<f32>(in.position, 1.0);

    let clip = camera.proj * camera.view * world_position;

    out.clip_position = clip;
    out.world_pos = world_position.xyz;

    out.tangent = vec4(normalize(normal_matrix * in.tangent.xyz), in.tangent.w);
    out.normal  = normalize(normal_matrix * in.normal);
    out.uv =  in.uv;

    // NDC
    out.ndc_xy = clip.xy / clip.w; // [-1, 1]

    out.entity_id = vec2<u32>(instance.entity_id_low, instance.entity_id_high);
    out.world_scale = compute_scale(model);

    return out;
}


/// Fragment shader
///

const True                   : u32 = 1;
const False                  : u32 = 0;

const AlphaMask              : u32 = 1; 

const DebugNone              : u32 = 0;
const TextureCoords0         : u32 = 1;
const TextureCoords1         : u32 = 2;
const DebugBaseColor         : u32 = 3; 
const DebugNormalTexture     : u32 = 4; 
const DebugGeometryNormal    : u32 = 5; 
const DebugGeometryTangent   : u32 = 6; 
const DebugGeometryBitangent : u32 = 7; 
const DebugGeometryTangentW  : u32 = 8;
const DebugShadingNormal     : u32 = 9;
const DebugMetallic          : u32 = 10; 
const DebugRoughness         : u32 = 11; 
const DebugEmissive          : u32 = 12; 
const DebugOcclusion         : u32 = 13; 
const DebugTransmission      : u32 = 14; 
const VolumeThickness        : u32 = 15; 
const SheenColor             : u32 = 16; 
const ShennRoughness         : u32 = 17; 

// Material
@group(1) @binding(0) var <uniform> material: Material;
@group(1) @binding(1) var tex_sampler: sampler;
@group(1) @binding(2) var albedo_map: texture_2d<f32>;
@group(1) @binding(3) var normal_map: texture_2d<f32>;
@group(1) @binding(4) var orm_map: texture_2d<f32>;         // Occlusion (R), Roughness (G), Metallic (B) https://github.com/KhronosGroup/glTF/issues/857
@group(1) @binding(5) var emissive_map: texture_2d<f32>; 
@group(1) @binding(6) var occlusion_map: texture_2d<f32>; 
@group(1) @binding(7) var transmission_map: texture_2d<f32>; 
@group(1) @binding(8) var volume_map: texture_2d<f32>; 

// Ibl 
@group(3) @binding(0) var ibl_sampler: sampler;
@group(3) @binding(1) var irradiance_map: texture_cube<f32>;
@group(3) @binding(2) var prefilter_map: texture_cube<f32>; // miplevels = 5
@group(3) @binding(3) var brdf_lut_map: texture_2d<f32>;
@group(3) @binding(4) var scene_sampler: sampler;           // transmission input scene sampler
@group(3) @binding(5) var scene_color: texture_2d<f32>; 

struct MaterialInfo {
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
    transmission: f32,
    thickness: f32,
}

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
    
    // Diffuse Term
    let kS = F;
    var kD = (vec3(1.0) - kS) * (1.0 - metallic);
    let irradiance = textureSample(irradiance_map, ibl_sampler, env_rotation * N).rgb;
    let  diffuse = irradiance * albedo * kD *  globals.ibl_intensity;

    // Specular Term
    var lod = roughness * MAX_REFLECTION_LOD;
    let prefiltered = textureSampleLevel(prefilter_map, ibl_sampler, env_rotation * R, lod).rgb;
    let brdf = textureSample(brdf_lut_map, ibl_sampler, vec2<f32>(NdotV, roughness)).rg;
    let specular = prefiltered * (F * brdf.x + brdf.y) * globals.ibl_intensity;

    result.diffuse  = diffuse;
    result.specular = specular;
    return result;
}


fn has_flag(flags: u32, index: u32) -> bool {
    return (flags & (1u << index)) != 0u;
}

fn get_uv(uv01: vec4<f32>, slot: u32) -> vec2<f32> {
    var uv = select(uv01.xy, uv01.zw, has_flag(material.coord_flags, slot));
    uv = uv_transform(material.texture_transform[slot], uv);
    return uv; 
}

fn uv_transform(m: mat3x3<f32>, uv: vec2<f32>) -> vec2<f32> {
    let tuv = m * vec3(uv, 1.0);
    return tuv.xy;
}

fn get_color(uv01: vec4<f32>) ->vec3<f32> {
    var albedo_color = material.color.rgb;
    if has_flag(material.texture_flags, COLOR_TEXTURE)  {
        let uv = get_uv(uv01, COLOR_TEXTURE);
        albedo_color *= textureSample(albedo_map, tex_sampler, uv).rgb;
    }
    return albedo_color;
}


fn get_alpha(uv01: vec4<f32>) ->f32 {
    var alpha = 0.0;
    if has_flag(material.texture_flags, COLOR_TEXTURE)  {
        let uv = get_uv(uv01, COLOR_TEXTURE);
        alpha = textureSample(albedo_map, tex_sampler, uv).a;
    }

    return alpha;
}

fn get_metallic(uv01: vec4<f32>) ->f32 {
    var metallic = material.metallic_factor;
    if has_flag(material.texture_flags, METAL_ROUGHNESS_TEXTURE) {
        let uv = get_uv(uv01, METAL_ROUGHNESS_TEXTURE);
        metallic *= textureSample(orm_map, tex_sampler, uv).b;
        metallic = select(0.0, metallic, metallic > 0.06); //select(false_value, true_value, condition)
    }
    return clamp(metallic, 0.0, 1.0);
}

fn get_roughness(uv01: vec4<f32>) ->f32 {
    var roughness = material.roughness_factor;
    if has_flag(material.texture_flags, METAL_ROUGHNESS_TEXTURE) {
        let uv = get_uv(uv01, METAL_ROUGHNESS_TEXTURE);
        roughness *= textureSample(orm_map, tex_sampler, uv).g;
    }
    return clamp(roughness, 0.04, 1.0);
}

fn get_occlusion(uv01: vec4<f32>) ->f32 {
    var ao:f32 = 1.0;
    if has_flag(material.texture_flags, OCCLUSION_TEXTURE) {
        var uv = get_uv(uv01, OCCLUSION_TEXTURE);
        let occlusion_texture = textureSample(occlusion_map, tex_sampler, uv).r;
        ao = 1.0 + material.occlusion_strength * (occlusion_texture - 1.0);
    }
    return ao;
}

fn get_emissive(uv01: vec4<f32>) ->vec3<f32> {
    var emissive = vec3<f32>(0.0);
    if has_flag(material.texture_flags, EMISSIVE_TEXTURE) {
        let uv = get_uv(uv01, EMISSIVE_TEXTURE);
        let emissive_texture = textureSample(emissive_map, tex_sampler, uv).rgb;
        emissive = emissive_texture * material.emissive.rgb;
    }
    return emissive;
}

fn get_normal_texture(uv01: vec4<f32>) ->vec3<f32> {
    var normal_ts = vec3<f32>(0.0);
    if has_flag(material.texture_flags, NORMAL_TEXTURE) {
        let uv = get_uv(uv01, NORMAL_TEXTURE);
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

fn get_thickness(uv01: vec4<f32>) ->f32 {
    var thickness = material.thickness_factor;
    if has_flag(material.texture_flags, VOLUME_TEXTURE) {
        let uv = get_uv(uv01, VOLUME_TEXTURE);
        thickness *= textureSample(volume_map, tex_sampler, uv).r;
    }
    thickness = max(thickness, 0.001);
    return thickness;
}

fn get_transmission(uv01: vec4<f32>) ->f32 {
    var transmission = material.transmission_factor;
    if has_flag(material.texture_flags, TRANSMISSION_TEXTURE) {
        let uv = get_uv(uv01, TRANSMISSION_TEXTURE);
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

fn sampleTransmission(
    world_pos: vec3<f32>,
    ndc_xy: vec2<f32>,
    N: vec3<f32>,   
    V: vec3<f32>,
    ior: f32,
    thickness: f32,
    roughness: f32,
    use_refraction: bool
) -> vec3<f32> {

    // UV base (no refraction)
    var uv = ndc_xy * 0.5 + vec2(0.5);
    uv.y = 1.0 - uv.y;

    let refraction_mask = select(0.0, 1.0, use_refraction);

    let eta = 1.0 / ior;
    let R = refract(-V, N, eta);

    let exit_pos = world_pos + R * thickness * refraction_mask;

    let clip = camera.proj * camera.view * vec4(exit_pos, 1.0);
    let ndc = clip.xy / clip.w;

    uv = ndc * 0.5 + vec2(0.5);
    uv.y = 1.0 - uv.y;
    
    uv = clamp(uv, vec2(0.001), vec2(0.999));

    let lod = roughness * MAX_SCENE_LOD;
    return textureSampleLevel(scene_color, scene_sampler, uv, lod).rgb;
}

// ---------------------------
// VOLUME (Beer-Lambert)
// ---------------------------
fn computeVolume(thickness: f32)->vec3<f32> {
    let att_color = material.attenuation_color;
    let sigma = -log(att_color) / max(material.attenuation_distance, 0.0001);
    let attenuation = exp(-sigma * thickness);

    return attenuation;
}

// ---------------------------
// DEBUG
// ---------------------------
fn computeChecker(ndc: vec2<f32>) -> vec3<f32> {
    let gray = 0.9;
    let scale = 20.0;

    let uv = ndc * 0.5 + vec2(0.5);
    let check = floor(uv * scale);
    let pattern = f32(i32(check.x + check.y) & 1);

    return vec3(gray + pattern * 0.1);
}

fn fresnel_schlick(F0: vec3<f32>, cos_theta: f32) -> vec3<f32> {
    let x = clamp(1.0 - cos_theta, 0.0, 1.0);
    return F0 + (vec3<f32>(1.0) - F0) * pow(x, 5.0);
}

fn dielectric_F0(ior: f32) -> f32 {
    let f = (ior - 1.0) / (ior + 1.0);
    return f * f;
}

fn compute_F0(albedo: vec3<f32>, metallic: f32, ior: f32) -> vec3<f32> {
    let dielectric = vec3<f32>(dielectric_F0(ior));
    return mix(dielectric, albedo, metallic);
}


fn compute_specular_transmission(V: vec3<f32>, Nws: vec3<f32>, albedo_color: vec3<f32>, metallic: f32, roughness: f32, ) ->vec3<f32> {
    let R = reflect(-V, Nws);
    let env_rotation = env_rotY();

    let lod = roughness * MAX_REFLECTION_LOD;
    let env_spec = textureSampleLevel(prefilter_map, ibl_sampler, env_rotation * R, lod).rgb;

    let NdotV = max(dot(Nws, V), 0.0);
    let F0 = compute_F0(albedo_color, metallic, material.ior);
    let F = fresnel_schlick(F0, NdotV);

    let specular_transmission = env_spec * F;

    return specular_transmission;
}

struct BRDFResult {
    diffuse: vec3<f32>,
    specular: vec3<f32>,
};

fn evalBRDF(
    N: vec3<f32>,
    V: vec3<f32>,
    mat: MaterialInfo,
    world_pos: vec3<f32>,
    ao: f32
) -> BRDFResult{

    let light_res = CalculateLight(N, V, mat.albedo, mat.metallic, mat.roughness, world_pos);
    let ambient_res = CalculateAmbient(N, V, mat.albedo, mat.metallic, mat.roughness);

    let diffuse  = light_res.diffuse + ambient_res.diffuse * ao;
    let specular = light_res.specular + ambient_res.specular * ao;

    return BRDFResult(diffuse, specular);
}

fn evalBTDF(
    world_pos: vec3<f32>, 
    ndc_xy: vec2<f32>, 
    N: vec3<f32>, 
    V: vec3<f32>, 
    mat: MaterialInfo,
    world_scale: f32, //  world scale
) -> vec3<f32> {

    let use_refraction = material.is_volume == True && mat.thickness > 0.0;

    let thickness = mat.thickness * world_scale;

    var transmission_color = sampleTransmission(
        world_pos, 
        ndc_xy,  
        N, 
        V, 
        material.ior, 
        thickness, 
        mat.roughness, 
        use_refraction
    );

    // tint
    transmission_color = mix(
        transmission_color,
        transmission_color * mat.albedo,
        mat.transmission
    );

    // volume
    if material.is_volume == True {
        transmission_color *= computeVolume(thickness);
    }

    return transmission_color;
}

struct FSOutput {
    @location(0) color : vec4<f32>,
    @location(1) entity_id : vec2<u32>,
}

fn CalculateSheenDirectLight(
    N: vec3<f32>,
    V: vec3<f32>,
    frag_pos: vec3<f32>,
    sheen_color: vec3<f32>,
    sheen_roughness: f32
) -> vec3<f32> {

    var result = vec3<f32>(0.0);

    if lights.enabled == False {
        return result;
    }

    for (var i: u32 = 0u; i < lights.count; i += 1u) {

        let light = lights.lights[i];

        if light.enabled == False {
            continue;
        }

        let L = normalize(light.position - frag_pos);
        let H = normalize(V + L);

        let NdotL = max(dot(N, L), 0.0);
        let NdotV = max(dot(N, V), 0.0);
        let NdotH = max(dot(N, H), 0.0);

        var radiance = vec3<f32>(0.0);

        if light.directional == True {
            radiance = light.color;
        } else {
            let d = length(light.position - frag_pos);
            let attenuation = 1.0 / (d * d);
            radiance = light.color * attenuation;
        }

        // fake Charlie-like distribution
        let distribution =
            pow(NdotH, mix(80.0, 2.0, sheen_roughness));

        // edge boost
        let fresnel =
            0.2 + 0.8 * pow(1.0 - NdotV, 5.0);

        let sheen =
            sheen_color *
            distribution *
            fresnel *
            NdotL;

        result += sheen * radiance;
    }

    return result;
}

fn CalculateSheenIBL(
    N: vec3<f32>,
    V: vec3<f32>,
    sheen_color: vec3<f32>,
    sheen_roughness: f32
) -> vec3<f32> {

    if globals.ibl_enable == False {
        return vec3(0.0);
    } 

    let env_rotation = env_rotY();

    let NdotV = max(dot(N, V), 0.0);

    let R = reflect(-V, N);

    let lod =
        mix(
            MAX_REFLECTION_LOD * 0.35,
            MAX_REFLECTION_LOD,
            sheen_roughness
        );

    let env =
        textureSampleLevel(
            prefilter_map,
            ibl_sampler,
            env_rotation * R,
            lod
        ).rgb;

    // cloth fresnel
    var fresnel = pow(1.0 - NdotV, 5.0);

    // small frontal visibility
    fresnel = max(fresnel, 0.025);

    // strong energy reduction
    let energy =
        mix(
            0.65,
            0.25,
            sheen_roughness
        );

    return env * sheen_color * fresnel * energy;
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
    let thickness = get_thickness(in.uv);

    let mat: MaterialInfo = MaterialInfo(
        albedo_color,
        metallic,
        roughness,
        transmission,
        thickness,
    );

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

    let brdf = evalBRDF(Nws, V, mat, in.world_pos, ao);
        
    var color = vec3<f32>();
    if material.is_trasmissive == True {
        let btdf = evalBTDF(
            in.world_pos,
            in.ndc_xy,
            Nws,
            V,
            mat,
            in.world_scale
        );
        let NdotV = max(dot(Nws, V), 0.0);

        let F0_dielectric = vec3<f32>(dielectric_F0(material.ior));
        let F = fresnel_schlick(F0_dielectric, NdotV);

        let spec_trans = compute_specular_transmission(
            V, Nws, albedo_color, metallic, roughness
        );

        color =
            brdf.specular + 
            brdf.diffuse * (1.0 - mat.transmission) +
            spec_trans * (1.0 - F) +
            btdf *  mat.transmission * (1.0 - F);

    } else { // Opaque
        color = brdf.specular + brdf.diffuse + emissive;

        if material.is_sheen == True {

            // direct light sheen
            let sheen_direct = CalculateSheenDirectLight(
                Nws,
                V,
                in.world_pos,
                material.sheen_color_factor,
                material.sheen_roughness_factor
            );
            // ibl sheen
            let sheen_ibl = CalculateSheenIBL(
                Nws,
                V,
                material.sheen_color_factor,
                material.sheen_roughness_factor
            );

            color += sheen_direct * 0.15;
            color += sheen_ibl * globals.ibl_intensity;
        }
    }

    switch globals.debug {
        case TextureCoords0         : { color = inverse_srgb(vec3(in.uv.xy, 0)); }
        case TextureCoords1         : { color = inverse_srgb(vec3(in.uv.zw, 0)); }
        case DebugBaseColor         : { color = albedo_color; }
        case DebugNormalTexture     : { color = inverse_srgb((normal_texture + 1.0) * 0.5);}
        case DebugGeometryNormal    : { color = inverse_srgb((N + 1.0) * 0.5 );}
        case DebugGeometryTangent   : { color = inverse_srgb((T + 1.0) * 0.5 );}
        case DebugGeometryBitangent : { color = inverse_srgb((B + 1.0) * 0.5 );}
        case DebugGeometryTangentW  : { color = inverse_srgb(vec3(in.tangent.w + 1.0) * 0.5);}
        case DebugShadingNormal     : { color = inverse_srgb((Nws + 1.0) * 0.5);}
        case DebugMetallic          : { color = inverse_srgb(vec3(metallic));}
        case DebugRoughness         : { color = inverse_srgb(vec3(roughness));}
        case DebugEmissive          : { color = emissive;}
        case DebugOcclusion         : { color = inverse_srgb(vec3(ao));}
        case DebugTransmission      : { color = vec3(transmission);}
        case VolumeThickness        : { color = inverse_srgb(vec3(thickness / material.thickness_factor));}
        case SheenColor             : { color = inverse_srgb(material.sheen_color_factor);}
        case ShennRoughness         : { color = inverse_srgb(vec3(material.sheen_roughness_factor));}
        default: {;} 
    }
    
    // attachement 0:
    out.color = vec4<f32>(color, 1.0);
    // attachement 1:
    out.entity_id =  in.entity_id;

    return out;
}


    // debug normal
    // color = -N * 0.5 + 0.5;
    // return vec4<f32>(normalize(in.normal) * 0.5 + 0.5, 1.0);

    // debug vettore vista
    // return vec4<f32>((V*0.5+0.5),1.0);
