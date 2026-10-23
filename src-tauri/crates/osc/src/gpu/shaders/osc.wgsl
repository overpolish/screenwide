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
  picked_colors: array<vec4<f32>, 2>, // an object kept, one removed
  picked_halo: vec4<f32>, // .x reach px, .y points of `picked_points`
  picked_points: array<vec4<f32>, 32>, // two points to a row, in pixels
  turn_colors: array<vec4<f32>, 3>, // the X, Y and Z rings
  // The rotation gizmo: rows 0-2 each ring's `u`, 3-5 each ring's `v`, in
  // pixels with the depth toward the viewer in `.z`; row 6 the middle in
  // pixels, the core's half width and the outline's width; row 7 the ring
  // held or -1, how far it is lit toward white, and how strongly far halves
  // show.
  turn_rings: array<vec4<f32>, 8>,
  window_cutout: vec4<f32>, // the window being picked, px: x, y, width, height
  window_shape: vec4<f32>, // .x corner radius px, .y fade, .z blur px (0: none), .w feather px
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
const WINDOW_BACKDROP_KIND: u32 = 64u;

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
  // A dense separable 9x9 Gaussian, sigma two taps two and a half texels
  // apart, reaches far enough that text under a plate turns to a wash
  // rather than legible shapes, and dense enough that its sample blocks
  // never show as pixels at fractional DPI.
  var weights = array<f32, 9>(0.0276, 0.0663, 0.1238, 0.1802, 0.2042, 0.1802, 0.1238, 0.0663,
                              0.0276);
  var blurred = vec4<f32>(0.0);
  for (var y = 0; y < 9; y++) {
    for (var x = 0; x < 9; x++) {
      let offset = vec2<f32>(f32(x - 4), f32(y - 4)) * osc.chrome_backdrop.zw * 2.5;
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

// The screen round the window being picked: the desktop frozen as picking
// starts, softened by the focus blur's disc (`disc_blur.wgsl`) from its mip
// levels, under the overlay's shade; the shade alone when the display could
// not be frozen. The window's rounded rectangle stays clear so the window
// shows live, the shade and the blur feathering into it together.
fn window_backdrop(pixel_position: vec2<f32>) -> vec4<f32> {
  let half_size = osc.window_cutout.zw * 0.5;
  let radius = min(osc.window_shape.x, min(half_size.x, half_size.y));
  let distance = rounded_distance(pixel_position - (osc.window_cutout.xy + half_size), half_size,
      radius);
  let feather = max(osc.window_shape.w, 1.0);
  let outside = smoothstep(-0.5 * feather, 0.5 * feather, distance) * osc.window_shape.y;
  let shade = osc.overlay_shade;
  if (osc.window_shape.z <= 0.0) {
    return vec4<f32>(shade.rgb, shade.a * outside);
  }
  // The backdrop is the display's own pixels, which the target covers.
  let size = vec2<f32>(textureDimensions(snapshot));
  let per = size / max(osc.chrome_backdrop.xy, vec2<f32>(1.0));
  let blurred = disc_blur(snapshot, linear_sampler, pixel_position * per,
      osc.window_shape.z * per.x, vec2<f32>(0.5), size - 0.5);
  return vec4<f32>(mix(blurred.rgb, shade.rgb, shade.a), outside);
}

// A quad's size in pixels, from how fast its unit uv changes across one. The
// rate is the length of each coordinate's screen gradient rather than its
// `fwidth`, so a quad turned off the screen's axes, such as a turned image's
// grip, measures its own sides rather than the box it fits in.
fn quad_pixels(uv: vec2<f32>) -> vec2<f32> {
  let x = dpdx(uv);
  let y = dpdy(uv);
  let rate = vec2<f32>(length(vec2<f32>(x.x, y.x)), length(vec2<f32>(x.y, y.y)));
  return 1.0 / max(rate, vec2<f32>(0.0001));
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

// A marching ant: a capsule a period long, `offset` measured from its middle
// in pixels along and across, of `radius` pixels round.
fn ant(along_offset: vec2<f32>, radius: f32) -> vec4<f32> {
  var offset = along_offset;
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

// The crop window's marching ants: a capsule a period, `uv` running along the
// edge in periods of 12 drawn pixels, so the pattern stays put wherever the
// edge starts.
fn marquee(kind: u32, uv: vec2<f32>) -> vec4<f32> {
  let horizontal = kind <= 8u;
  let longitudinal = select(uv.y, uv.x, horizontal);
  let transverse = select(uv.x, uv.y, horizontal);
  let period = 1.0 / max(fwidth(longitudinal), 0.0001);
  let thickness = 1.0 / max(fwidth(transverse), 0.0001);
  return ant(
    vec2<f32>((fract(longitudinal) - 0.5) * period, (transverse - 0.5) * thickness),
    thickness * 0.5,
  );
}

// A side of a turned marquee band: the same ants along a side that need not
// run along the pixel grid. Its quad is padded a pixel either side, as
// `soft_line`'s is, for the ants' rims to anti-alias inside it; `uv.x` runs
// along it in periods of 12 drawn pixels and `uv.y` across it. Each rate is
// its gradient's length, as `quad_pixels` measures, since `fwidth` would
// overstate it on a slanted side.
fn soft_marquee(uv: vec2<f32>) -> vec4<f32> {
  let period = 1.0 / max(length(vec2<f32>(dpdx(uv.x), dpdy(uv.x))), 0.0001);
  let thickness = quad_pixels(uv).y;
  return ant(
    vec2<f32>((fract(uv.x) - 0.5) * period, (uv.y - 0.5) * thickness),
    thickness * 0.5 - 1.0,
  );
}

// The snap guides' colours: the canvas's own lines in one, another object's
// in the other.
fn guide_color(object: bool) -> vec4<f32> {
  if (object) {
    return select(vec4<f32>(0.055, 0.647, 0.914, 1.0), vec4<f32>(0.008, 0.518, 0.780, 1.0), is_light());
  }
  return vec4<f32>(0.918, 0.702, 0.031, 1.0);
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

// A line that need not run along the pixel grid, as a turned frame's sides
// and a turned layer's snap chrome do: its quad is padded one pixel either
// side so its edges can be anti-aliased, with `uv.y` running across it. 50
// is the line in the controls' fill and 51 the halo in their outline; 61 and
// 62 are a guide's, in the canvas's colour and another object's.
fn soft_line(kind: u32, uv: vec2<f32>) -> vec4<f32> {
  let dimensions = quad_pixels(uv);
  let across = abs(uv.y - 0.5) * dimensions.y;
  let half = dimensions.y * 0.5 - 1.0;
  let coverage = clamp(half - across + 0.5, 0.0, 1.0);
  if (coverage <= 0.0) {
    discard;
  }
  var color = select(osc.control_colors[0], osc.control_colors[1], kind == 51u);
  if (kind == 61u || kind == 62u) {
    color = guide_color(kind == 62u);
  }
  color.a *= coverage;
  return color;
}

// The rotation gizmo's three rings, drawn by one quad over all of them, as
// Keyframeless draws them: each pixel measures its distance to the whole of
// every ring at once, so a ring is one band with no seams where pieces of it
// would overlap. A ring fades from full strength on its near half to dim on
// its far half through the points level with the middle, and the three are
// laid over one another by how near the pixel's point of each is. Each ring
// draws a coloured core in a dark outline; the one the pointer holds is lit
// toward white.
const TURN_RING_KIND: u32 = 60u;

// Points each ring is measured through: fine enough that the polyline lies
// within a tenth of a pixel of the circle at the rings' size.
const TURN_RING_SAMPLES: u32 = 64u;

// The depth, as a share of the radius, over which a ring fades from its near
// half's strength to its far half's.
const TURN_RING_FADE: f32 = 0.15;

// Ring `ring` with its nearest point `distance` pixels away at `depth`
// toward the viewer, in straight alpha.
fn turn_ring_paint(ring: u32, distance: f32, depth: f32) -> vec4<f32> {
  let core = osc.turn_rings[6].z;
  let outline_width = osc.turn_rings[6].w;
  let coverage = clamp(core + outline_width - distance + 0.5, 0.0, 1.0);
  var fill = osc.turn_colors[ring].rgb;
  if (f32(ring) == osc.turn_rings[7].x) {
    fill = mix(fill, vec3<f32>(1.0), osc.turn_rings[7].y);
  }
  let outline = clamp(distance - core + 0.5, 0.0, 1.0);
  var color = mix(vec4<f32>(fill, 1.0), vec4<f32>(0.0, 0.0, 0.0, 0.75), outline);
  let near = smoothstep(-TURN_RING_FADE, TURN_RING_FADE, depth);
  color.a *= coverage * mix(osc.turn_rings[7].z, 1.0, near);
  return color;
}

// `top` laid over `under`, both in straight alpha.
fn turn_over(top: vec4<f32>, under: vec4<f32>) -> vec4<f32> {
  let alpha = top.a + under.a * (1.0 - top.a);
  if (alpha <= 0.0) {
    return vec4<f32>(0.0);
  }
  return vec4<f32>((top.rgb * top.a + under.rgb * under.a * (1.0 - top.a)) / alpha, alpha);
}

fn turn_rings(at: vec2<f32>) -> vec4<f32> {
  let point = at - osc.turn_rings[6].xy;
  // Each ring's nearest distance and the depth there.
  var nearest = array<vec2<f32>, 3>(vec2<f32>(1e9, 0.0), vec2<f32>(1e9, 0.0),
                                    vec2<f32>(1e9, 0.0));
  for (var ring = 0u; ring < 3u; ring++) {
    let u = osc.turn_rings[ring].xyz;
    let v = osc.turn_rings[ring + 3u].xyz;
    var previous = u;
    for (var index = 1u; index <= TURN_RING_SAMPLES; index++) {
      let t = 6.2831853 * f32(index) / f32(TURN_RING_SAMPLES);
      let current = cos(t) * u + sin(t) * v;
      let edge = current.xy - previous.xy;
      let along = clamp(dot(point - previous.xy, edge) / max(dot(edge, edge), 1e-6), 0.0, 1.0);
      let distance = length(point - (previous.xy + along * edge));
      if (distance < nearest[ring].x) {
        nearest[ring] = vec2<f32>(distance, mix(previous.z, current.z, along));
      }
      previous = current;
    }
  }
  // Farthest first, so the nearest ring is laid on top.
  var order = array<u32, 3>(0u, 1u, 2u);
  for (var round = 0u; round < 2u; round++) {
    for (var index = 0u; index < 2u - round; index++) {
      if (nearest[order[index]].y > nearest[order[index + 1u]].y) {
        let swap = order[index];
        order[index] = order[index + 1u];
        order[index + 1u] = swap;
      }
    }
  }
  var color = vec4<f32>(0.0);
  for (var index = 0u; index < 3u; index++) {
    let ring = order[index];
    color = turn_over(turn_ring_paint(ring, nearest[ring].x, nearest[ring].y), color);
  }
  if (color.a <= 0.0) {
    discard;
  }
  return color;
}

// How strongly the halo round a chosen object picked in the camera shows:
// as the halo round an annotation does (`annotation_hover_alpha`).
const PICKED_HALO_ALPHA: f32 = 0.24;

// An object picked in the camera: 52 one kept, 53 one removed, a disc in its
// colour inside a one-pixel ring in the controls' outline, which keeps it
// apart from whatever the camera shows behind it. 54 a kept one's halo and
// 58 a removed one's: a soft disc of its colour round the chosen one, as an
// annotation is haloed.
fn picked(kind: u32, uv: vec2<f32>) -> vec4<f32> {
  let dimensions = quad_pixels(uv);
  let distance = length((uv - 0.5) * dimensions);
  let radius = min(dimensions.x, dimensions.y) * 0.5;
  let aa = max(fwidth(distance), 0.0001);
  let edge = distance - radius;
  let coverage = clamp(0.5 - edge / aa, 0.0, 1.0);
  if (coverage <= 0.0) {
    discard;
  }
  if (kind == 54u || kind == 58u) {
    var halo = osc.picked_colors[select(0u, 1u, kind == 58u)];
    halo.a *= PICKED_HALO_ALPHA * coverage;
    return halo;
  }
  let outline_mix = clamp(0.5 + (edge + 1.0) / aa, 0.0, 1.0);
  var color = mix(osc.picked_colors[kind - 52u], osc.control_colors[1], outline_mix);
  color.a *= coverage;
  return color;
}

// A line painted over an object picked in the camera, padded a pixel either
// side for its edges to be anti-aliased across, as `soft_line`: 55 in a kept
// object's colour, 56 in a removed one's.
fn picked_line(kind: u32, uv: vec2<f32>) -> vec4<f32> {
  let dimensions = quad_pixels(uv);
  let across = abs(uv.y - 0.5) * dimensions.y;
  let half = dimensions.y * 0.5 - 1.0;
  let coverage = clamp(half - across + 0.5, 0.0, 1.0);
  if (coverage <= 0.0) {
    discard;
  }
  var color = osc.picked_colors[kind - 55u];
  color.a *= coverage;
  return color;
}

fn picked_point(index: u32) -> vec2<f32> {
  let pair = osc.picked_points[index / 2u];
  return select(pair.xy, pair.zw, index % 2u == 1u);
}

// The halo round the line painted over the chosen object picked in the
// camera, 57 a kept one's and 59 a removed one's: every pixel within reach
// of the line, measured to the nearest point of it, as an annotation's halo
// is, so it is one smooth band with round ends rather than pieces laid over
// one another. Its quad covers the line and the reach round it.
fn picked_line_halo(kind: u32, at: vec2<f32>) -> vec4<f32> {
  let count = u32(osc.picked_halo.y);
  var nearest = 1e20;
  if (count > 0u) {
    nearest = length(at - picked_point(0u));
  }
  for (var index = 1u; index < count; index++) {
    let start = picked_point(index - 1u);
    let along = picked_point(index) - start;
    let share = clamp(dot(at - start, along) / max(dot(along, along), 1e-6), 0.0, 1.0);
    nearest = min(nearest, length(at - (start + along * share)));
  }
  let coverage = clamp(osc.picked_halo.x - nearest + 0.5, 0.0, 1.0);
  if (coverage <= 0.0) {
    discard;
  }
  var halo = osc.picked_colors[select(0u, 1u, kind == 59u)];
  halo.a *= PICKED_HALO_ALPHA * coverage;
  return halo;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
  let kind = input.kind;
  let uv = input.uv;
  if (kind == LENS_KIND) {
    return lens(input.position.xy);
  }
  if (kind == 52u || kind == 53u || kind == 54u || kind == 58u) {
    return picked(kind, uv);
  }
  if (kind == 55u || kind == 56u) {
    return picked_line(kind, uv);
  }
  if (kind == 57u || kind == 59u) {
    return picked_line_halo(kind, input.position.xy);
  }
  if (kind == TURN_RING_KIND) {
    return turn_rings(input.position.xy);
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
  if (kind == WINDOW_BACKDROP_KIND) {
    return window_backdrop(input.position.xy);
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
  // 61 and 62 are the guides' soft twins, drawn on a turned layer.
  if (kind == 50u || kind == 51u || kind == 61u || kind == 62u) {
    return soft_line(kind, uv);
  }
  if (kind == 63u) {
    return soft_marquee(uv);
  }
  if (kind == 4u || kind == 5u) {
    return guide_color(kind == 5u);
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
