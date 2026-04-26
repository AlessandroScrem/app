/// Vertex shader

struct Camera {
    view_pos: vec3<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    screen_size: vec2<f32>,
};

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

@group(0) @binding(0) var<uniform> camera: Camera;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {

    var out: VertexOutput;
    let world_position = vec4<f32>(vertex.position, 1.0);

    let clip_position = camera.proj * camera.view * world_position;
    // fix: (error pixel coverage)
    out.clip_position = clip_position + 1e-4;
    out.color = vertex.color;

    return out;
}

/// Fragment shader
///
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(input.color, 1.0);
}