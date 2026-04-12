@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var dst: texture_storage_2d<rgba8unorm, write>;

@compute
@workgroup_size(16, 16, 1)
fn cs_main(
    @builtin(global_invocation_id) gid: vec3<u32>,
) {
    let dst_dim = textureDimensions(dst);

    // confronto in u32 (safe)
    if (gid.x >= dst_dim.x || gid.y >= dst_dim.y) {
        return;
    }

    let dst_pos = vec2<i32>(gid.xy);
    let src_pos = dst_pos * 2;

    let p00 = textureLoad(src, src_pos, 0);
    let p01 = textureLoad(src, src_pos + vec2<i32>(0, 1), 0);
    let p10 = textureLoad(src, src_pos + vec2<i32>(1, 0), 0);
    let p11 = textureLoad(src, src_pos + vec2<i32>(1, 1), 0);

    let color = (p00 + p01 + p10 + p11) * 0.25;

    textureStore(dst, dst_pos, color);
}