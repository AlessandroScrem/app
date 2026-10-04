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
@group(0) @binding(0) var depth_sampler: sampler;
@group(0) @binding(1) var depth_tex: texture_depth_2d;

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let d = textureSample(depth_tex, depth_sampler, uv);

    // depth:
    // near objects = 0
    // far objets   = 1
    return vec4<f32>(d, d, d, 1.0);

    // let c = pow(1.0 - d, 0.1);
//   return vec4<f32>(vec3(c), 1.0);


}
