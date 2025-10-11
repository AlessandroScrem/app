struct Globals {
    ibl_enable: u32,
    skybox_enable: u32,
    exposure: f32,
    tonemap_filter: u32,
    selected_entity_id_low: u32,
    selected_entity_id_high: u32,
};

@group(1) @binding(1) var<uniform> globals: Globals;

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
  let uv  = 0.5 * (p + vec2<f32>(1.0, 1.0));
  out.uv = vec2<f32>(uv.x, 1.0 - uv.y);
  return out;
}


@group(0) @binding(1) var t_mask: texture_2d<u32>;

fn is_selected(color: vec2<u32>) -> bool {
    return color.r == globals.selected_entity_id_low && color.g == globals.selected_entity_id_high;
}



// Funzione che calcola l'edge factor sfumato
fn compute_outline_soft(uv: vec2<f32>, thickness: f32, softness: f32) -> f32 {
    let tex_size = vec2<f32>(textureDimensions(t_mask, 0));
    let texel_coord = vec2<i32>(uv * tex_size);

    let idColor: vec2<u32> = textureLoad(t_mask, texel_coord, 0).rg;

    if (!is_selected(idColor)) {
        return 0.0; // non selezionato → niente outline
    }

    let texelSize = (1.0 / tex_size) * thickness;

    var diffCount: f32 = 0.0;

    // Pixel vicini (cardinali + diagonali)
    let neighbor0 = textureLoad(t_mask, vec2<i32>(clamp(uv + vec2(-texelSize.x,  0.0), vec2(0.0), vec2(1.0)) * tex_size), 0).rg;
    let neighbor1 = textureLoad(t_mask, vec2<i32>(clamp(uv + vec2( texelSize.x,  0.0), vec2(0.0), vec2(1.0)) * tex_size), 0).rg;
    let neighbor2 = textureLoad(t_mask, vec2<i32>(clamp(uv + vec2(0.0, -texelSize.y), vec2(0.0), vec2(1.0)) * tex_size), 0).rg;
    let neighbor3 = textureLoad(t_mask, vec2<i32>(clamp(uv + vec2(0.0,  texelSize.y), vec2(0.0), vec2(1.0)) * tex_size), 0).rg;
    let neighbor4 = textureLoad(t_mask, vec2<i32>(clamp(uv + vec2(-texelSize.x, -texelSize.y), vec2(0.0), vec2(1.0)) * tex_size), 0).rg;
    let neighbor5 = textureLoad(t_mask, vec2<i32>(clamp(uv + vec2( texelSize.x, -texelSize.y), vec2(0.0), vec2(1.0)) * tex_size), 0).rg;
    let neighbor6 = textureLoad(t_mask, vec2<i32>(clamp(uv + vec2(-texelSize.x,  texelSize.y), vec2(0.0), vec2(1.0)) * tex_size), 0).rg;
    let neighbor7 = textureLoad(t_mask, vec2<i32>(clamp(uv + vec2( texelSize.x,  texelSize.y), vec2(0.0), vec2(1.0)) * tex_size), 0).rg;

    if (neighbor0.x != idColor.x || neighbor0.y != idColor.y) { diffCount += 1.0; }
    if (neighbor1.x != idColor.x || neighbor1.y != idColor.y) { diffCount += 1.0; }
    if (neighbor2.x != idColor.x || neighbor2.y != idColor.y) { diffCount += 1.0; }
    if (neighbor3.x != idColor.x || neighbor3.y != idColor.y) { diffCount += 1.0; }
    if (neighbor4.x != idColor.x || neighbor4.y != idColor.y) { diffCount += 1.0; }
    if (neighbor5.x != idColor.x || neighbor5.y != idColor.y) { diffCount += 1.0; }
    if (neighbor6.x != idColor.x || neighbor6.y != idColor.y) { diffCount += 1.0; }
    if (neighbor7.x != idColor.x || neighbor7.y != idColor.y) { diffCount += 1.0; }

    return smoothstep(0.0, 8.0 * softness, diffCount);
}

const outlineThickness: f32 = 3.0;
const outlineSoftness: f32 = 0.0;
const outlineColor = vec4(1.0, 0.214, 0, 1.0);

fn linear_to_srgb(c: vec3<f32>) -> vec3<f32> {
    var out: vec3<f32>;
    out.r = select(1.055 * pow(c.r, 1.0/2.4) - 0.055, 12.92 * c.r, c.r <= 0.0031308);
    out.g = select(1.055 * pow(c.g, 1.0/2.4) - 0.055, 12.92 * c.g, c.g <= 0.0031308);
    out.b = select(1.055 * pow(c.b, 1.0/2.4) - 0.055, 12.92 * c.b, c.b <= 0.0031308);
    return out;
}

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        select(pow((c.r + 0.055) / 1.055, 2.4), c.r / 12.92, c.r <= 0.04045),
        select(pow((c.g + 0.055) / 1.055, 2.4), c.g / 12.92, c.g <= 0.04045),
        select(pow((c.b + 0.055) / 1.055, 2.4), c.b / 12.92, c.b <= 0.04045)
    );
}

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {


    let edgeFactor = compute_outline_soft(uv, outlineThickness, outlineSoftness);
    if (edgeFactor == 0.0) {
        discard;
    }

    // let outlineColor =  vec4<f32>(srgb_to_linear(outlineColor.rgb), 1.0);

    return mix(vec4<f32>(0.0), outlineColor, edgeFactor);
}
