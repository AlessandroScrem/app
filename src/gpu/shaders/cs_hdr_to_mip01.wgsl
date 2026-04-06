@group(0) @binding(0) var src_hdr: texture_2d<f32>;
@group(0) @binding(1) var dst_mip0: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var dst_mip1: texture_storage_2d<rgba8unorm, write>;

fn tonemap(c: vec3<f32>) -> vec3<f32> {
    // semplice e veloce (puoi cambiarlo)
    return c / (1.0 + c); // Reinhard
}

@compute @workgroup_size(8, 8)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let base = vec2<i32>(gid.xy * 2u);

    let dim0 = vec2<i32>(textureDimensions(dst_mip0).xy);
    if (base.x >= dim0.x || base.y >= dim0.y) {
        return;
    }

    // --- legge 2x2 HDR ---
    let c00 = textureLoad(src_hdr, base + vec2<i32>(0, 0), 0).rgb;
    let c10 = textureLoad(src_hdr, base + vec2<i32>(1, 0), 0).rgb;
    let c01 = textureLoad(src_hdr, base + vec2<i32>(0, 1), 0).rgb;
    let c11 = textureLoad(src_hdr, base + vec2<i32>(1, 1), 0).rgb;

    // --- tonemap ---
    let t00 = tonemap(c00);
    let t10 = tonemap(c10);
    let t01 = tonemap(c01);
    let t11 = tonemap(c11);

    // --- scrivi mip 0 ---
    textureStore(dst_mip0, base + vec2<i32>(0, 0), vec4<f32>(t00, 1.0));
    textureStore(dst_mip0, base + vec2<i32>(1, 0), vec4<f32>(t10, 1.0));
    textureStore(dst_mip0, base + vec2<i32>(0, 1), vec4<f32>(t01, 1.0));
    textureStore(dst_mip0, base + vec2<i32>(1, 1), vec4<f32>(t11, 1.0));

    // --- genera mip 1 (media) ---
    let avg = (t00 + t10 + t01 + t11) * 0.25;

    let mip1_coord = vec2<i32>(gid.xy);
    let dim1 = vec2<i32>(textureDimensions(dst_mip1).xy);

    if (mip1_coord.x < dim1.x && mip1_coord.y < dim1.y) {
        textureStore(dst_mip1, mip1_coord, vec4<f32>(avg, 1.0));
    }
}