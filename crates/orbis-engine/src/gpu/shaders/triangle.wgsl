struct VSOut {
  @builtin(position) pos : vec4<f32>,
  @location(0) uv : vec2<f32>,
};

// Fullscreen triangle
@vertex
fn vs_main(@builtin(vertex_index) vid : u32) -> VSOut {
  var pos = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -3.0),
    vec2<f32>( 3.0,  1.0),
    vec2<f32>(-1.0,  1.0)
  );
  var out : VSOut;
  let p = pos[vid];
  out.pos = vec4<f32>(p, 0.0, 1.0);
  // mappa da NDC a UV
  out.uv  = 0.5 * (p + vec2<f32>(1.0, 1.0));
  return out;
}


@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {

  return vec4<f32>(uv.x, uv.y, 0.0, 1.0);
}
