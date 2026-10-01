// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#ifndef SCREENWIDE_OSC_GPU_MACOS_SHADER_FRAGMENT_H
#define SCREENWIDE_OSC_GPU_MACOS_SHADER_FRAGMENT_H

/// Every OSC vertex kind resolved to a colour: chrome, handles, controls,
/// OCR boxes, and the ruler artwork all branch on `region_osc_out.kind`.
#define SCREENWIDE_REGION_OSC_SHADER_FRAGMENT @R"METAL(
fragment float4 region_osc_fragment(
    region_osc_out in [[stage_in]],
    constant uint &light_mode [[buffer(0)]],
    constant float4 &magnifier_box [[buffer(1)]],
    constant region_osc_action_palette &actions [[buffer(2)]],
    constant region_osc_control_palette &controls [[buffer(3)]],
    constant region_osc_ocr_palette &ocr [[buffer(4)]],
    constant float4 &overlay_shade [[buffer(5)]],
    constant region_osc_ruler_palette &ruler [[buffer(6)]],
    constant float4 &ruler_sample [[buffer(7)]],
    constant float4 &ruler_animation [[buffer(8)]],
    texture2d<float> label [[texture(0)]],
    texture2d<float> secondary_label [[texture(1)]],
    texture2d<float> icons [[texture(2)]],
    texture2d<float> snapshot [[texture(3)]]) {
  constexpr sampler label_sampler(filter::linear, address::clamp_to_edge);
  if (in.kind == 33) {
    // Zoomed in, the frozen desktop's pixels are magnified as pixels, the
    // way a zoomed screenshot shows them, rather than smeared by bilinear
    // filtering. Magnification is read off the uv derivatives: fewer than
    // one texel per screen pixel. At one-to-one the linear sample resolves
    // the same texels.
    constexpr sampler pixel_sampler(filter::nearest, address::clamp_to_edge);
    float2 texels_per_pixel =
        fwidth(in.uv) * float2(snapshot.get_width(), snapshot.get_height());
    bool magnified = max(texels_per_pixel.x, texels_per_pixel.y) < 0.999;
    return magnified ? snapshot.sample(pixel_sampler, in.uv)
                     : snapshot.sample(label_sampler, in.uv);
  }
  if (in.kind == 34 || in.kind == 35) {
    float2 dimensions = 1.0 / max(fwidth(in.uv), float2(0.0001));
    float width = in.kind == 34 ? max(ruler_animation.z, 1.0) : 3.0;
    float margin = width * 0.5 + 1.0;
    float2 half_size = dimensions * 0.5;
    float2 centerline_half = max(half_size - margin, float2(0.0));
    float2 point = abs((in.uv - 0.5) * dimensions) - centerline_half;
    float distance = length(max(point, 0.0)) +
                     min(max(point.x, point.y), 0.0);
    float ring_distance = abs(distance) - width * 0.5;
    float aa = max(fwidth(distance), 0.5);
    float coverage = clamp(0.5 - ring_distance / aa, 0.0, 1.0);
    if (coverage <= 0.0) discard_fragment();
    float4 color = ruler.primary;
    color.a *= coverage * (in.kind == 34 ? ruler_animation.y : 0.32);
    return color;
  }
  if (in.kind >= 39 && in.kind <= 41) {
    float2 pixel_radius = 1.0 / max(fwidth(in.uv), float2(0.0001));
    float radius = (pixel_radius.x + pixel_radius.y) * 0.5;
    float2 local = in.uv * radius;
    float width = in.kind == 40 ? max(ruler_animation.z, 1.0) : 1.0;
    float half_width = width * 0.5;
    float radial = abs(length(local) - radius) - half_width;
    float quadrant = max(-local.x, -local.y);
    float arc_distance = max(radial, quadrant);
    float endpoint_x = length(local - float2(radius, 0.0)) - half_width;
    float endpoint_y = length(local - float2(0.0, radius)) - half_width;
    float distance = min(arc_distance, min(endpoint_x, endpoint_y));
    float aa = max(fwidth(distance), 0.5);
    float coverage = clamp(0.5 - distance / aa, 0.0, 1.0);
    if (in.kind == 41) {
      float along = atan2(max(local.y, 0.0), max(local.x, 0.0)) * radius;
      float phase = fmod(along, 7.0);
      float pattern_aa = max(fwidth(along), 0.5);
      float dash = 1.0 - smoothstep(4.0 - pattern_aa,
                                   4.0 + pattern_aa, phase);
      coverage *= dash;
    }
    if (coverage <= 0.0) discard_fragment();
    float4 color = ruler.primary;
    color.a *= coverage * (in.kind == 40 ? ruler_animation.y
                                         : in.kind == 41 ? 0.7 : 1.0);
    return color;
  }
  if (magnifier_box.z > 0.0) {
    // Same radius and the same feathered coverage the compute pass rounds the
    // loupe with. The scene keeps drawing wherever the loupe is not fully
    // opaque, so the two edges blend into each other instead of meeting at a
    // hard step.
    float radius = max(magnifier_box.z / 12.0, 1.0);
    float2 half_size = magnifier_box.zw * 0.5;
    float2 local = abs(in.position.xy - (magnifier_box.xy + half_size)) -
                   (half_size - radius);
    float distance = length(max(local, 0.0)) +
                     min(max(local.x, local.y), 0.0) - radius;
    float coverage = 1.0 - smoothstep(-0.5, 0.5, distance);
    if (coverage >= 1.0) discard_fragment();
  }
  if (in.kind == 37) {
    float4 sampled = secondary_label.sample(label_sampler, in.uv);
    if (sampled.a <= 0.002) discard_fragment();
    return float4(sampled.rgb / sampled.a,
                  sampled.a * ruler_animation.w);
  }
  if (in.kind == 11 || in.kind == 48) {
    float4 sampled = label.sample(label_sampler, in.uv);
    if (sampled.a <= 0.002) discard_fragment();
    float opacity = in.kind == 48 ? 1.0 - ruler_animation.w : 1.0;
    return float4(sampled.rgb / sampled.a, sampled.a * opacity);
  }
  if (in.kind == 28) return ruler.primary;
  if (in.kind >= 42 && in.kind <= 44) {
    float4 color = ruler.primary;
    color.a *= in.kind == 42 ? 0.45 : in.kind == 43 ? 0.85 : 0.30;
    return color;
  }
  if (in.kind == 36) return ruler.info;
  if (in.kind == 38) {
    float4 color = ruler.info;
    color.a *= ruler_animation.y;
    return color;
  }
  if (in.kind == 31) {
    float4 color = ruler.primary;
    color.a *= 0.32;
    return color;
  }
  if (in.kind == 32) {
    float4 color = ruler.primary;
    color.a *= ruler_animation.y;
    return color;
  }
  if (in.kind == 29) {
    float2 dimensions = 1.0 / max(fwidth(in.uv), float2(0.0001));
    float2 half_size = dimensions * 0.5;
    float radius = min(4.0, min(half_size.x, half_size.y));
    float2 point = abs((in.uv - 0.5) * dimensions) - (half_size - radius);
    float distance = length(max(point, 0.0)) +
                     min(max(point.x, point.y), 0.0) - radius;
    float aa = max(fwidth(distance), 0.0001);
    float coverage = clamp(0.5 - distance / aa, 0.0, 1.0);
    if (coverage <= 0.0) discard_fragment();
    float4 color = ruler_sample;
    color.a *= coverage * (1.0 - ruler_animation.x) *
        (1.0 - ruler_animation.w);
    return color;
  }
  if (in.kind == 30) {
    float4 color = actions.secondary;
    color.a *= ruler_animation.x * (1.0 - ruler_animation.w);
    return color;
  }
  if (in.kind >= 22 && in.kind <= 26) {
    float cell = float(in.kind - 21);
    float2 atlas_uv = float2((cell + in.uv.x) / 6.0, in.uv.y);
    float coverage = icons.sample(label_sampler, atlas_uv).r;
    if (coverage <= 0.002) discard_fragment();
    float4 color = actions.secondary;
    color.a *= coverage;
    return color;
  }
  if (in.kind >= 12 && in.kind <= 14) {
    // Material-backed OSC controls have one semantic radius owner: their
    // native surface. Keeping the Metal fill rectangular prevents a second,
    // height-derived radius from diverging on composed controls such as the
    // two-row ruler loupe.
    float4 color = in.kind == 13 ? actions.secondary : actions.primary;
    return color;
  }
  if (in.kind == 6) return overlay_shade;
  if (in.kind == 45) {
    // A crop corner: uv is the distance from the arc centre in radii, so what
    // lies outside the arc is what the rounded layer will not keep.
    float distance = length(in.uv);
    float aa = max(fwidth(distance), 0.0001);
    float outside = clamp((distance - 1.0) / aa + 0.5, 0.0, 1.0);
    if (outside <= 0.0) discard_fragment();
    float4 color = overlay_shade;
    color.a *= outside;
    return color;
  }
  if (in.kind >= 17 && in.kind <= 20) {
    float2 dimensions = 1.0 / max(fwidth(in.uv), float2(0.0001));
    float2 half_size = dimensions * 0.5;
    float radius = min(4.0, min(half_size.x, half_size.y));
    float2 point = abs((in.uv - 0.5) * dimensions) - (half_size - radius);
    float distance = length(max(point, 0.0)) +
                     min(max(point.x, point.y), 0.0) - radius;
    float aa = max(fwidth(distance), 0.0001);
    float coverage = clamp(0.5 - distance / aa, 0.0, 1.0);
    if (coverage <= 0.0) discard_fragment();
    float4 fill = in.kind == 20 ? ocr.selection_fill
        : in.kind == 19 ? ocr.error_fill
        : in.kind == 18 ? ocr.qr_fill : ocr.primary_fill;
    float4 outline = in.kind == 20 ? ocr.selection_outline
        : in.kind == 19 ? ocr.error_outline
        : in.kind == 18 ? ocr.qr_outline : ocr.primary_outline;
    float outline_width = in.kind == 18 ? 2.0 : 1.0;
    float outline_mix =
        clamp(0.5 + (distance + outline_width) / aa, 0.0, 1.0);
    float4 color = mix(fill, outline, outline_mix);
    color.a *= coverage;
    return color;
  }
  if (in.kind >= 7 && in.kind <= 10) {
    bool horizontal = in.kind <= 8;
    float longitudinal = horizontal ? in.uv.x : in.uv.y;
    float transverse = horizontal ? in.uv.y : in.uv.x;
    float period = 1.0 / max(fwidth(longitudinal), 0.0001);
    float thickness = 1.0 / max(fwidth(transverse), 0.0001);
    float2 point = float2((fract(longitudinal) - 0.5) * period,
                          (transverse - 0.5) * thickness);
    float radius = thickness * 0.5;
    float half_segment = 3.0;
    point.x -= clamp(point.x, -half_segment, half_segment);
    float distance = length(point) - radius;
    float aa = max(fwidth(distance), 0.0001);
    float coverage = clamp(0.5 - distance / aa, 0.0, 1.0);
    if (coverage <= 0.0) discard_fragment();
    // A single outer capsule owns both colors. Keeping anti-aliasing at the
    // exterior prevents the fill from compositing over the one-pixel ring.
    float outline_mix = clamp(0.5 + (distance + 1.0) / aa, 0.0, 1.0);
    float4 color = mix(controls.fill, controls.outline, outline_mix);
    color.a *= coverage;
    return color;
  }
  if (in.kind == 3 || in.kind == 16) {
    float2 dimensions = 1.0 / max(fwidth(in.uv), float2(0.0001));
    float2 point = (in.uv - 0.5) * dimensions;
    float radius = max(min(dimensions.x, dimensions.y) * 0.5 - 1.0, 0.0);
    if (in.kind == 16) {
      bool horizontal = dimensions.x >= dimensions.y;
      float half_segment = abs(dimensions.x - dimensions.y) * 0.5;
      if (horizontal)
        point.x -= clamp(point.x, -half_segment, half_segment);
      else
        point.y -= clamp(point.y, -half_segment, half_segment);
    }
    float distance = length(point) - radius;
    float aa = max(fwidth(distance), 0.0001);
    float coverage = clamp(0.5 - distance / aa, 0.0, 1.0);
    if (coverage <= 0.0) discard_fragment();
    float outline_mix = clamp(0.5 + (distance + 1.0) / aa, 0.0, 1.0);
    float4 color = mix(controls.fill, controls.outline, outline_mix);
    color.a *= coverage;
    return color;
  }
  float coverage = 1.0;
  bool guide = in.kind == 4 || in.kind == 5;
  if (!guide && (in.kind & 1) != 0) {
    float2 dimensions = 1.0 / max(fwidth(in.uv), float2(0.0001));
    float2 point = (in.uv - 0.5) * dimensions;
    float radius = max(min(dimensions.x, dimensions.y) * 0.5 - 1.0, 0.0);
    float edge = length(point) - radius;
    float aa = max(fwidth(edge), 0.5);
    coverage = 1.0 - smoothstep(-aa, aa, edge);
    if (coverage <= 0.0) discard_fragment();
  }
  if (guide) {
    if (in.kind == 5)
      return light_mode != 0 ? float4(0.008, 0.518, 0.780, 1.0)
                             : float4(0.055, 0.647, 0.914, 1.0);
    return float4(0.918, 0.702, 0.031, 1.0);
  }
  bool halo = in.kind >= 2;
  float4 color = halo ? controls.outline : controls.fill;
  color.a *= coverage;
  return color;
}
)METAL"

#endif
