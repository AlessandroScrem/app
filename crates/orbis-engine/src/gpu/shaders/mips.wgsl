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



@group(0) @binding(0) var src_sampler: sampler;
@group(0) @binding(1) var src_tex: texture_2d<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_size = vec2<f32>(textureDimensions(src_tex));
    let texel = 1.0 / tex_size;

    let uv = in.uv;


    // box filter 2x2
    let c =
        textureSample(src_tex, src_sampler, uv) +
        textureSample(src_tex, src_sampler, uv + vec2(texel.x, 0.0)) +
        textureSample(src_tex, src_sampler, uv + vec2(0.0, texel.y)) +
        textureSample(src_tex, src_sampler, uv + texel);

    return c * 0.25;
}