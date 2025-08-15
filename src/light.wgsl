/// Vertex shader

struct Camera {
    view_pos: vec3<f32>,
    view_proj: mat4x4<f32>,
};

struct Model {
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
};

@group(0) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(0) var<uniform> model: Model;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};


@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {

    var out: VertexOutput;
    let world_position = model.model * vec4<f32>(vertex.position, 1.0);

    out.clip_position = camera.view_proj * world_position;

    return out;
}

/// Fragment shader
///

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {

    return vec4<f32>(1.0, 1.0, 0.0, 1.0); // giallo
}