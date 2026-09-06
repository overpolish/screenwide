// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

static NSString *const ScreenwideRegionOscMetalSource = @R"(
#include <metal_stdlib>
using namespace metal;

struct region_magnifier_state {
  uint active;
  uint pane_index;
  uint layer_id;
  uint sample_camera;
  uint edges;
  uint light_mode;
  float sample_u;
  float sample_v;
  float source_min_u;
  float source_min_v;
  float source_max_u;
  float source_max_v;
  int box_x;
  int box_y;
  uint box_width;
  uint box_height;
};

kernel void region_magnifier(
    const device uchar4 *source [[buffer(0)]],
    texture2d<float, access::read_write> output [[texture(0)]],
    constant uint2 &source_dimensions [[buffer(1)]],
    constant region_magnifier_state &magnifier [[buffer(2)]],
    uint2 gid [[thread_position_in_grid]]) {
  if (magnifier.active == 0 || gid.x >= magnifier.box_width ||
      gid.y >= magnifier.box_height || any(source_dimensions == 0)) return;
  int2 output_point = int2(magnifier.box_x, magnifier.box_y) + int2(gid);
  if (any(output_point < 0) || output_point.x >= int(output.get_width()) ||
      output_point.y >= int(output.get_height())) return;
  float2 box_size = float2(magnifier.box_width, magnifier.box_height);
  float2 local = float2(gid) + 0.5;
  float radius = 4.0;
  float2 half_size = box_size * 0.5;
  float2 rounded = abs(local - half_size) - (half_size - radius);
  float distance = length(max(rounded, 0.0)) +
                   min(max(rounded.x, rounded.y), 0.0) - radius;
  if (distance > 0.0) return;
  float2 source_center = float2(magnifier.sample_u, magnifier.sample_v) *
                         float2(source_dimensions);
  float2 source_point = source_center +
      (local / box_size - 0.5) * 40.0;
  int2 sample_point = int2(floor(source_point));
  float2 sample_uv = source_point / float2(source_dimensions);
  bool in_source = all(sample_point >= 0) &&
      all(sample_point < int2(source_dimensions)) &&
      all(sample_uv >= float2(magnifier.source_min_u,
                              magnifier.source_min_v)) &&
      all(sample_uv <= float2(magnifier.source_max_u,
                              magnifier.source_max_v));
  float4 pixel = in_source
      ? float4(source[uint(sample_point.y) * source_dimensions.x +
                      uint(sample_point.x)]) / 255.0
      : float4(0.15, 0.15, 0.16, 1.0);
  bool shade = ((magnifier.edges & 1u) != 0u && local.x < half_size.x) ||
               ((magnifier.edges & 2u) != 0u && local.x >= half_size.x) ||
               ((magnifier.edges & 4u) != 0u && local.y < half_size.y) ||
               ((magnifier.edges & 8u) != 0u && local.y >= half_size.y);
  if (shade) {
    float3 shade_color = magnifier.light_mode != 0
        ? float3(0.0) : float3(1.0);
    pixel.rgb = mix(pixel.rgb, shade_color, 0.1);
  }
  if (distance > -1.0) pixel = float4(0.15, 0.15, 0.16, 1.0);
  output.write(pixel, uint2(output_point));
}

struct region_osc_vertex {
  float2 position;
  float2 uv;
  uint kind;
  uint padding;
};

struct region_osc_out {
  float4 position [[position]];
  float2 uv;
  uint kind;
};

struct region_osc_control_palette {
  float4 fill;
  float4 outline;
};

struct region_osc_action_palette {
  float4 primary;
  float4 secondary;
};

struct region_osc_ocr_palette {
  float4 primary_fill;
  float4 primary_outline;
  float4 qr_fill;
  float4 qr_outline;
  float4 error_fill;
  float4 error_outline;
  float4 selection_fill;
  float4 selection_outline;
};

struct region_osc_ruler_palette {
  float4 primary;
  float4 info;
};

vertex region_osc_out region_osc_vertex_main(
    const device region_osc_vertex *vertices [[buffer(0)]],
    uint index [[vertex_id]]) {
  region_osc_out out;
  out.position = float4(vertices[index].position, 0.0, 1.0);
  out.uv = vertices[index].uv;
  out.kind = vertices[index].kind;
  return out;
}

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
  if (in.kind == 33) return snapshot.sample(label_sampler, in.uv);
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
    float2 half_size = magnifier_box.zw * 0.5;
    float2 local = abs(in.position.xy - (magnifier_box.xy + half_size)) -
                   (half_size - 4.0);
    float distance = length(max(local, 0.0)) +
                     min(max(local.x, local.y), 0.0) - 4.0;
    if (distance <= 0.0) discard_fragment();
  }
  if (in.kind == 37) {
    float4 sampled = secondary_label.sample(label_sampler, in.uv);
    if (sampled.a <= 0.002) discard_fragment();
    return float4(sampled.rgb / sampled.a,
                  sampled.a * ruler_animation.w);
  }
  if (in.kind == 11 || in.kind == 15 || in.kind == 48) {
    float4 sampled = in.kind == 15
        ? secondary_label.sample(label_sampler, in.uv)
        : label.sample(label_sampler, in.uv);
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
  if (in.kind >= 17 && in.kind <= 20) {
    float2 dimensions = 1.0 / max(fwidth(in.uv), float2(0.0001));
    float2 half_size = dimensions * 0.5;
    float radius = min(2.0, min(half_size.x, half_size.y));
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
    float outline_width = in.kind == 17 || in.kind == 18 ? 2.0 : 1.0;
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
)";
