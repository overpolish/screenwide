// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Every OSC vertex kind resolved to a colour: chrome, handles, controls, OCR
// boxes, the ruler artwork and the magnifier lens all branch on `kind`.
//
// The kind is a flat per-vertex value, so WGSL's uniformity analysis treats
// every branch on it as divergent and would reject the screen-space
// derivatives most kinds measure their own size with. A triangle draws one
// kind, so its quads take one branch and the derivatives are well defined.
diagnostic(off, derivative_uniformity);

// One uniform for every palette and state the OSC draws with. Every member is
// a `vec4` row, so Rust's `RenderConstants` lays out the same way.
struct OscGpu {
  light_mode: vec4<u32>, // .x: 0 dark, 1 light
  magnifier_box: vec4<f32>, // x/y/width/height in physical pixels
  action_fills: array<vec4<f32>, 2>, // primary, secondary/foreground
  control_colors: array<vec4<f32>, 2>, // fill, outline
  ocr_colors: array<vec4<f32>, 8>, // the first eight fields of the OCR palette
  overlay_shade: vec4<f32>,
  ruler_colors: array<vec4<f32>, 2>, // primary, info
  ruler_sample: vec4<f32>, // picked pixel colour
  ruler_animation: vec4<f32>, // copied, hover alpha, hover width px, tolerance
  magnifier_source: vec4<f32>, // source width/height in pixels
  magnifier_sample: vec4<f32>, // anchor u/v inside the source
  magnifier_source_range: vec4<f32>, // min u/v, max u/v
  magnifier_flags: vec4<u32>, // edges bitmask, active
  // macOS masked its floating control surfaces with the material view's
  // corner radius; without one, the plate owns its radius and outline.
  chrome: vec4<f32>, // .x: radius; .y: material emphasis
  chrome_outline: vec4<f32>, // plate outline; alpha 0 means no outline
  chrome_backdrop: vec4<f32>, // viewport width/height, source texel width/height
  chrome_source: vec4<f32>, // snapshot UV x/y/width/height after pan/zoom
}

@group(0) @binding(0) var<uniform> osc: OscGpu;
@group(0) @binding(1) var label: texture_2d<f32>;
@group(0) @binding(2) var secondary_label: texture_2d<f32>;
@group(0) @binding(3) var icons: texture_2d<f32>;
@group(0) @binding(4) var snapshot: texture_2d<f32>;
@group(0) @binding(5) var magnifier_texture: texture_2d<f32>;
@group(0) @binding(6) var linear_sampler: sampler;
@group(0) @binding(7) var point_sampler: sampler;

// The layout every OSC vertex builder emits, the macOS one included.
struct VertexIn {
  @location(0) position: vec2<f32>,
  @location(1) uv: vec2<f32>,
  @location(2) kind: u32,
}

struct VertexOut {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) @interpolate(flat) kind: u32,
}

// The magnifier lens. Crop corners already own 45.
const LENS_KIND: u32 = 49u;

// Positions arrive in NDC: the vertex builder does the pixel-to-clip mapping
// on the CPU, so there is no transform here.
@vertex
fn vs_main(input: VertexIn) -> VertexOut {
  var output: VertexOut;
  output.position = vec4<f32>(input.position, 0.0, 1.0);
  output.uv = input.uv;
  output.kind = input.kind;
  return output;
}

fn rounded_distance(offset: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
  let local = abs(offset) - (half_size - radius);
  return length(max(local, vec2<f32>(0.0))) + min(max(local.x, local.y), 0.0) - radius;
}

fn is_light() -> bool {
  return osc.light_mode.x != 0u;
}

fn material_backdrop(pixel_position: vec2<f32>) -> vec4<f32> {
  if (osc.chrome_backdrop.z <= 0.0 || osc.chrome_backdrop.w <= 0.0) {
    return select(vec4<f32>(0.12, 0.12, 0.13, 1.0), vec4<f32>(0.92, 0.92, 0.93, 1.0), is_light());
  }
  let screen_uv = pixel_position / max(osc.chrome_backdrop.xy, vec2<f32>(1.0));
  let uv = osc.chrome_source.xy + screen_uv * osc.chrome_source.zw;
  // A dense separable 5x5 Gaussian keeps edges smooth at fractional DPI. A
  // sparse nine-tap grid exposes its sample blocks as visible pixels.
  var weights = array<f32, 5>(0.06136, 0.24477, 0.38774, 0.24477, 0.06136);
  var blurred = vec4<f32>(0.0);
  for (var y = 0; y < 5; y++) {
    for (var x = 0; x < 5; x++) {
      let offset = vec2<f32>(f32(x - 2), f32(y - 2)) * osc.chrome_backdrop.zw * 2.0;
      blurred += textureSampleLevel(snapshot, linear_sampler, uv + offset, 0.0) * weights[x] *
          weights[y];
    }
  }
  // Mica is an opaque, wallpaper-derived material rather than a transparent
  // blur. Recreate its two exposed controls: compress luminosity and
  // saturation, then apply the theme tint. The frozen desktop stays
  // recognisable while dark controls get a low-luminance base.
  let luminance = dot(blurred.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
  let muted = mix(vec3<f32>(luminance), blurred.rgb, 0.58);
  let luminosity_base = select(
      mix(muted, vec3<f32>(0.045, 0.047, 0.052), 0.58),
      mix(muted, vec3<f32>(0.90, 0.90, 0.91), 0.40),
      is_light());
  let tint = select(vec3<f32>(0.070, 0.072, 0.080), vec3<f32>(0.965, 0.965, 0.975), is_light());
  return vec4<f32>(mix(luminosity_base, tint, select(0.36, 0.30, is_light())), 1.0);
}

// A quad's size in pixels, from how fast its unit uv changes across one.
fn quad_pixels(uv: vec2<f32>) -> vec2<f32> {
  return 1.0 / max(fwidth(uv), vec2<f32>(0.0001));
}

// The lens the constants describe; `lens.wgsl` draws it.
fn osc_lens() -> Lens {
  return Lens(osc.magnifier_box, osc.magnifier_source, osc.magnifier_sample,
              osc.magnifier_source_range,
              vec4<u32>(osc.magnifier_flags.xy, osc.light_mode.x, 0u));
}

// Source-over blending takes the straight alpha, so the lens quad fades into
// the scene it is drawn over.
fn lens(position: vec2<f32>) -> vec4<f32> {
  let color = lens_color(osc_lens(), position, magnifier_texture, point_sampler);
  if (color.a <= 0.0) {
    discard;
  }
  return color;
}

fn chrome_plate(uv: vec2<f32>, position: vec2<f32>) -> vec4<f32> {
  let dimensions = quad_pixels(uv);
  let half_size = dimensions * 0.5;
  let radius = min(osc.chrome.x, min(half_size.x, half_size.y));
  let distance = rounded_distance((uv - 0.5) * dimensions, half_size, radius);
  let aa = max(fwidth(distance), 0.0001);
  let coverage = clamp(0.5 - distance / aa, 0.0, 1.0);
  if (coverage <= 0.0) {
    discard;
  }
  var outline_mix = 0.0;
  if (osc.chrome_outline.a > 0.002) {
    outline_mix = clamp(0.5 + (distance + 1.0) / aa, 0.0, 1.0);
  }
  var material = material_backdrop(position);
  let material_tint = select(
      vec3<f32>(0.035, 0.037, 0.042), vec3<f32>(0.92, 0.92, 0.93), is_light());
  material = vec4<f32>(
      mix(material.rgb, material_tint, saturate(osc.chrome.y) * 0.22), material.a);
  let control = osc.action_fills[0];
  var color = vec4<f32>(mix(material.rgb, control.rgb, control.a), 1.0);
  color = mix(color, osc.chrome_outline, outline_mix * osc.chrome_outline.a);
  color.a *= coverage;
  return color;
}

fn ruler_ring(kind: u32, uv: vec2<f32>) -> vec4<f32> {
  let dimensions = quad_pixels(uv);
  let width = select(3.0, max(osc.ruler_animation.z, 1.0), kind == 34u);
  let margin = width * 0.5 + 1.0;
  let half_size = dimensions * 0.5;
  let centerline_half = max(half_size - margin, vec2<f32>(0.0));
  let offset = abs((uv - 0.5) * dimensions) - centerline_half;
  let distance = length(max(offset, vec2<f32>(0.0))) + min(max(offset.x, offset.y), 0.0);
  let ring_distance = abs(distance) - width * 0.5;
  let aa = max(fwidth(distance), 0.5);
  let coverage = clamp(0.5 - ring_distance / aa, 0.0, 1.0);
  if (coverage <= 0.0) {
    discard;
  }
  var color = osc.ruler_colors[0];
  color.a *= coverage * select(0.32, osc.ruler_animation.y, kind == 34u);
  return color;
}

fn ruler_arc(kind: u32, uv: vec2<f32>) -> vec4<f32> {
  let pixel_radius = quad_pixels(uv);
  let radius = (pixel_radius.x + pixel_radius.y) * 0.5;
  let local = uv * radius;
  let width = select(1.0, max(osc.ruler_animation.z, 1.0), kind == 40u);
  let half_width = width * 0.5;
  let radial = abs(length(local) - radius) - half_width;
  let quadrant = max(-local.x, -local.y);
  let arc_distance = max(radial, quadrant);
  let endpoint_x = length(local - vec2<f32>(radius, 0.0)) - half_width;
  let endpoint_y = length(local - vec2<f32>(0.0, radius)) - half_width;
  let distance = min(arc_distance, min(endpoint_x, endpoint_y));
  let aa = max(fwidth(distance), 0.5);
  var coverage = clamp(0.5 - distance / aa, 0.0, 1.0);
  if (kind == 41u) {
    let along = atan2(max(local.y, 0.0), max(local.x, 0.0)) * radius;
    let phase = along % 7.0;
    let pattern_aa = max(fwidth(along), 0.5);
    coverage *= 1.0 - smoothstep(4.0 - pattern_aa, 4.0 + pattern_aa, phase);
  }
  if (coverage <= 0.0) {
    discard;
  }
  var color = osc.ruler_colors[0];
  var alpha = 1.0;
  if (kind == 40u) {
    alpha = osc.ruler_animation.y;
  } else if (kind == 41u) {
    alpha = 0.7;
  }
  color.a *= coverage * alpha;
  return color;
}

// Straight colour out of a premultiplied label texture.
fn label_color(sampled: vec4<f32>, opacity: f32) -> vec4<f32> {
  if (sampled.a <= 0.002) {
    discard;
  }
  return vec4<f32>(sampled.rgb / sampled.a, sampled.a * opacity);
}

fn picked_swatch(uv: vec2<f32>) -> vec4<f32> {
  let dimensions = quad_pixels(uv);
  let half_size = dimensions * 0.5;
  let radius = min(4.0, min(half_size.x, half_size.y));
  let distance = rounded_distance((uv - 0.5) * dimensions, half_size, radius);
  let aa = max(fwidth(distance), 0.0001);
  let coverage = clamp(0.5 - distance / aa, 0.0, 1.0);
  if (coverage <= 0.0) {
    discard;
  }
  var color = osc.ruler_sample;
  color.a *= coverage * (1.0 - osc.ruler_animation.x) * (1.0 - osc.ruler_animation.w);
  return color;
}

fn icon(kind: u32, uv: vec2<f32>) -> vec4<f32> {
  let cell = f32(kind - 21u);
  let atlas_uv = vec2<f32>((cell + uv.x) / 6.0, uv.y);
  let coverage = textureSampleLevel(icons, linear_sampler, atlas_uv, 0.0).r;
  if (coverage <= 0.002) {
    discard;
  }
  var color = osc.action_fills[1];
  color.a *= coverage;
  return color;
}

fn crop_corner(uv: vec2<f32>) -> vec4<f32> {
  // uv is the distance from the arc centre in radii, so what lies outside the
  // arc is what the rounded layer will not keep.
  let corner_distance = length(uv);
  let corner_aa = max(fwidth(corner_distance), 0.0001);
  let outside = clamp((corner_distance - 1.0) / corner_aa + 0.5, 0.0, 1.0);
  if (outside <= 0.0) {
    discard;
  }
  var color = osc.overlay_shade;
  color.a *= outside;
  return color;
}

fn ocr_box(kind: u32, uv: vec2<f32>) -> vec4<f32> {
  let dimensions = quad_pixels(uv);
  let half_size = dimensions * 0.5;
  let radius = min(4.0, min(half_size.x, half_size.y));
  let distance = rounded_distance((uv - 0.5) * dimensions, half_size, radius);
  let aa = max(fwidth(distance), 0.0001);
  let coverage = clamp(0.5 - distance / aa, 0.0, 1.0);
  if (coverage <= 0.0) {
    discard;
  }
  // Kinds 17-20 read fill/outline pairs 0-1, 2-3, 4-5 and 6-7.
  let pair = (kind - 17u) * 2u;
  let fill = osc.ocr_colors[pair];
  let outline = osc.ocr_colors[pair + 1u];
  let outline_width = select(1.0, 2.0, kind == 18u);
  let outline_mix = clamp(0.5 + (distance + outline_width) / aa, 0.0, 1.0);
  var color = mix(fill, outline, outline_mix);
  color.a *= coverage;
  return color;
}

// The crop window's marching ants: a capsule a period, `uv` running along the
// edge in periods of 12 drawn pixels, so the pattern stays put wherever the
// edge starts.
fn marquee(kind: u32, uv: vec2<f32>) -> vec4<f32> {
  let horizontal = kind <= 8u;
  let longitudinal = select(uv.y, uv.x, horizontal);
  let transverse = select(uv.x, uv.y, horizontal);
  let period = 1.0 / max(fwidth(longitudinal), 0.0001);
  let thickness = 1.0 / max(fwidth(transverse), 0.0001);
  var offset = vec2<f32>((fract(longitudinal) - 0.5) * period, (transverse - 0.5) * thickness);
  let radius = thickness * 0.5;
  let half_segment = 3.0;
  offset.x -= clamp(offset.x, -half_segment, half_segment);
  let distance = length(offset) - radius;
  let aa = max(fwidth(distance), 0.0001);
  let coverage = clamp(0.5 - distance / aa, 0.0, 1.0);
  if (coverage <= 0.0) {
    discard;
  }
  // A single outer capsule owns both colours. Anti-aliasing at the exterior
  // keeps the fill from compositing over the one-pixel ring.
  let outline_mix = clamp(0.5 + (distance + 1.0) / aa, 0.0, 1.0);
  var color = mix(osc.control_colors[0], osc.control_colors[1], outline_mix);
  color.a *= coverage;
  return color;
}

fn handle(kind: u32, uv: vec2<f32>) -> vec4<f32> {
  let dimensions = quad_pixels(uv);
  var offset = (uv - 0.5) * dimensions;
  let radius = max(min(dimensions.x, dimensions.y) * 0.5 - 1.0, 0.0);
  if (kind == 16u) {
    let half_segment = abs(dimensions.x - dimensions.y) * 0.5;
    if (dimensions.x >= dimensions.y) {
      offset.x -= clamp(offset.x, -half_segment, half_segment);
    } else {
      offset.y -= clamp(offset.y, -half_segment, half_segment);
    }
  }
  let distance = length(offset) - radius;
  let aa = max(fwidth(distance), 0.0001);
  let coverage = clamp(0.5 - distance / aa, 0.0, 1.0);
  if (coverage <= 0.0) {
    discard;
  }
  let outline_mix = clamp(0.5 + (distance + 1.0) / aa, 0.0, 1.0);
  var color = mix(osc.control_colors[0], osc.control_colors[1], outline_mix);
  color.a *= coverage;
  return color;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
  let kind = input.kind;
  let uv = input.uv;
  if (kind == LENS_KIND) {
    return lens(input.position.xy);
  }
  if (kind == 33u) {
    // A frozen desktop is an opaque backing plane. Capture APIs may leave
    // alpha unspecified, and exposing it through a premultiplied composition
    // would reveal the live desktop beneath the snapshot. Zoomed in, the
    // desktop's pixels are magnified as pixels, the way a zoomed screenshot
    // shows them, rather than smeared by bilinear filtering. Magnification is
    // read off the uv derivatives: fewer than one texel per drawn pixel. At
    // one-to-one the linear sample resolves the same texels.
    let texels_per_pixel = fwidth(uv) * vec2<f32>(textureDimensions(snapshot));
    let magnified = max(texels_per_pixel.x, texels_per_pixel.y) < 0.999;
    var sampled = textureSampleLevel(snapshot, linear_sampler, uv, 0.0);
    if (magnified) {
      sampled = textureSampleLevel(snapshot, point_sampler, uv, 0.0);
    }
    return vec4<f32>(sampled.rgb, 1.0);
  }
  if (kind == 46u) {
    // The rounded chrome plate. Kinds 12-14 stay rectangular because on macOS
    // the surface owned the radius; this one owns its own.
    return chrome_plate(uv, input.position.xy);
  }
  if (kind == 47u) {
    // Chrome text: the texture carries white coverage and the tint is the
    // portable control foreground.
    let coverage = textureSampleLevel(label, linear_sampler, uv, 0.0).a;
    if (coverage <= 0.002) {
      discard;
    }
    var color = osc.action_fills[1];
    color.a *= coverage;
    return color;
  }
  if (kind == 34u || kind == 35u) {
    return ruler_ring(kind, uv);
  }
  if (kind >= 39u && kind <= 41u) {
    return ruler_arc(kind, uv);
  }
  // Everything below gives way to the lens, which is drawn over it.
  if (lens_coverage(osc_lens(), input.position.xy) >= 1.0) {
    discard;
  }
  if (kind == 37u) {
    let sampled = textureSampleLevel(secondary_label, linear_sampler, uv, 0.0);
    return label_color(sampled, osc.ruler_animation.w);
  }
  if (kind == 11u || kind == 48u) {
    let sampled = textureSampleLevel(label, linear_sampler, uv, 0.0);
    return label_color(sampled, select(1.0, 1.0 - osc.ruler_animation.w, kind == 48u));
  }
  if (kind == 15u) {
    return label_color(textureSampleLevel(secondary_label, linear_sampler, uv, 0.0), 1.0);
  }
  if (kind == 28u) {
    return osc.ruler_colors[0];
  }
  if (kind >= 42u && kind <= 44u) {
    var color = osc.ruler_colors[0];
    var alpha = 0.30;
    if (kind == 42u) {
      alpha = 0.45;
    } else if (kind == 43u) {
      alpha = 0.85;
    }
    color.a *= alpha;
    return color;
  }
  if (kind == 36u) {
    return osc.ruler_colors[1];
  }
  if (kind == 38u) {
    var color = osc.ruler_colors[1];
    color.a *= osc.ruler_animation.y;
    return color;
  }
  if (kind == 31u) {
    var color = osc.ruler_colors[0];
    color.a *= 0.32;
    return color;
  }
  if (kind == 32u) {
    var color = osc.ruler_colors[0];
    color.a *= osc.ruler_animation.y;
    return color;
  }
  if (kind == 29u) {
    return picked_swatch(uv);
  }
  if (kind == 30u) {
    var color = osc.action_fills[1];
    color.a *= osc.ruler_animation.x * (1.0 - osc.ruler_animation.w);
    return color;
  }
  if (kind >= 22u && kind <= 26u) {
    return icon(kind, uv);
  }
  if (kind >= 12u && kind <= 14u) {
    // Material-backed OSC controls have one semantic radius owner: their
    // native surface. A rectangular fill keeps a second, height-derived
    // radius from diverging on composed controls such as the ruler loupe.
    return select(osc.action_fills[0], osc.action_fills[1], kind == 13u);
  }
  if (kind == 6u) {
    return osc.overlay_shade;
  }
  if (kind == 45u) {
    return crop_corner(uv);
  }
  if (kind >= 17u && kind <= 20u) {
    return ocr_box(kind, uv);
  }
  if (kind >= 7u && kind <= 10u) {
    return marquee(kind, uv);
  }
  if (kind == 3u || kind == 16u) {
    return handle(kind, uv);
  }
  let guide = kind == 4u || kind == 5u;
  if (guide) {
    if (kind == 5u) {
      return select(vec4<f32>(0.055, 0.647, 0.914, 1.0), vec4<f32>(0.008, 0.518, 0.780, 1.0), is_light());
    }
    return vec4<f32>(0.918, 0.702, 0.031, 1.0);
  }
  var coverage = 1.0;
  if ((kind & 1u) != 0u) {
    let dimensions = quad_pixels(uv);
    let offset = (uv - 0.5) * dimensions;
    let radius = max(min(dimensions.x, dimensions.y) * 0.5 - 1.0, 0.0);
    let edge = length(offset) - radius;
    let aa = max(fwidth(edge), 0.5);
    coverage = 1.0 - smoothstep(-aa, aa, edge);
    if (coverage <= 0.0) {
      discard;
    }
  }
  var color = select(osc.control_colors[0], osc.control_colors[1], kind >= 2u);
  color.a *= coverage;
  return color;
}
