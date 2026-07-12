const MAX_LIGHTS             : u32 = 64;

/// Vertex shader

struct Light {
    view_proj: mat4x4<f32>,

    color: vec3<f32>,
    directional: u32,

    position: vec3<f32>,
    cast_shadow: u32,
    
    entity_id_low: u32,
    entity_id_high: u32,
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

@group(0) @binding(0) var<uniform> light  : Light;

@vertex
fn vs_main(
    in: VertexInput,
    instance: InstanceInput,
) ->  @builtin(position) vec4<f32> {

    let model = mat4_from_instance(instance);

    return light.view_proj * model * vec4<f32>(in.position, 1.0);
}


@fragment
fn fs_main() {}

