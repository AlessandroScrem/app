@group(0) @binding(0) var src_hdr: texture_2d<f32>;
@group(0) @binding(1) var src_sampler: sampler;
@group(0) @binding(2) var dst_mip0: texture_storage_2d<rgba8unorm, write>;

fn tonemap(c: vec3<f32>) -> vec3<f32> {
    // semplice e veloce (puoi cambiarlo)
    return c / (1.0 + c); // Reinhard
}

@compute @workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dim = textureDimensions(dst_mip0);
    if (gid.x >= dim.x || gid.y >= dim.y) { return; }

    let uv = (vec2<f32>(gid.xy) + 0.5) / vec2<f32>(dim);
    let color = textureSampleLevel(src_hdr, src_sampler, uv, 0.0).rgb;

    let ldr = tonemap(color);
    textureStore(dst_mip0, vec2<i32>(gid.xy), vec4<f32>(ldr, 1.0));
}