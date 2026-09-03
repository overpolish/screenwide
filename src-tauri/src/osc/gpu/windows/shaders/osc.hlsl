// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Metal pushed nine fragment slots (b0-b8); D3D11 takes one buffer, so the
// fields keep their Metal declaration order and are padded to float4 rows.
cbuffer OscGpu : register(b0) {
  uint4 light_mode;          // .x: 0 dark, 1 light
  float4 magnifier_box;      // x/y/width/height in physical pixels
  float4 action_fills[2];    // primary, secondary/foreground
  float4 control_colors[2];  // fill, outline
  float4 ocr_colors[8];      // the first eight fields of the OCR palette
  float4 overlay_shade;
  float4 ruler_colors[2];    // primary, info
  float4 ruler_sample;       // picked pixel color
  float4 ruler_animation;    // copied, hover alpha, hover width px, tolerance
  float4 magnifier_source;   // source width/height in pixels
  float4 magnifier_sample;   // anchor u/v inside the source
  float4 magnifier_source_range; // min u/v, max u/v
  uint4 magnifier_flags;     // edges bitmask, active
  // Appended for the OCR chrome. macOS masked its floating control surfaces
  // with the material view's cornerRadius; without one, the plate owns its own
  // radius and hairline outline.
  float4 chrome;             // .x: radius; .y: material emphasis
  float4 chrome_outline;     // plate outline; alpha 0 means no outline
  float4 chrome_backdrop;    // viewport width/height, source texel width/height
  float4 chrome_source;      // snapshot UV x/y/width/height after pan/zoom
  float4 outlined_label;     // .x: halo sample radius in physical pixels
};

Texture2D label : register(t0);
Texture2D secondary_label : register(t1);
Texture2D icons : register(t2);
Texture2D snapshot : register(t3);
Texture2D magnifier_texture : register(t4);
SamplerState linear_sampler : register(s0);
SamplerState point_sampler : register(s1);

struct VertexIn {
  float2 position : POSITION;
  float2 uv : TEXCOORD0;
  float2 aux : TEXCOORD1;
  uint kind : TEXCOORD2;
};

struct VertexOut {
  float4 position : SV_Position;
  float2 uv : TEXCOORD0;
  nointerpolation float2 aux : TEXCOORD1;
  nointerpolation uint kind : TEXCOORD2;
};

// Positions arrive in NDC: the vertex builder does the pixel-to-clip mapping
// on the CPU, so there is no transform here.
VertexOut vs_main(VertexIn input) {
  VertexOut output;
  output.position = float4(input.position, 0.0, 1.0);
  output.uv = input.uv;
  output.aux = input.aux;
  output.kind = input.kind;
  return output;
}

float rounded_distance(float2 offset, float2 half_size, float radius) {
  float2 local = abs(offset) - (half_size - radius);
  return length(max(local, 0.0)) + min(max(local.x, local.y), 0.0) - radius;
}

float4 material_backdrop(float2 pixel_position) {
  if (chrome_backdrop.z <= 0.0 || chrome_backdrop.w <= 0.0) {
    return light_mode.x != 0u ? float4(0.92, 0.92, 0.93, 1.0)
                              : float4(0.12, 0.12, 0.13, 1.0);
  }
  float2 screen_uv = pixel_position / max(chrome_backdrop.xy, 1.0);
  float2 uv = chrome_source.xy + screen_uv * chrome_source.zw;
  // A dense separable 5x5 Gaussian keeps edges smooth at fractional DPI. The
  // old sparse nine-tap grid exposed its sample blocks as visible pixels.
  static const float weights[5] = { 0.06136, 0.24477, 0.38774, 0.24477, 0.06136 };
  float4 blurred = 0.0;
  [unroll]
  for (int y = 0; y < 5; y++) {
    [unroll]
    for (int x = 0; x < 5; x++) {
      float2 offset = float2(x - 2, y - 2) * chrome_backdrop.zw * 2.0;
      blurred += snapshot.SampleLevel(linear_sampler, uv + offset, 0) * weights[x] * weights[y];
    }
  }
  // Mica is an opaque, wallpaper-derived material rather than a transparent
  // blur. Recreate its two exposed controls here: first compress luminosity
  // and saturation, then apply the theme tint. This keeps the frozen desktop
  // recognisable while giving dark controls the missing low-luminance base.
  float luminance = dot(blurred.rgb, float3(0.2126, 0.7152, 0.0722));
  float3 muted = lerp(luminance.xxx, blurred.rgb, 0.58);
  float3 luminosity_base = light_mode.x != 0u
      ? lerp(muted, float3(0.90, 0.90, 0.91), 0.40)
      : lerp(muted, float3(0.045, 0.047, 0.052), 0.58);
  float3 tint = light_mode.x != 0u ? float3(0.965, 0.965, 0.975)
                                   : float3(0.070, 0.072, 0.080);
  blurred.rgb = lerp(luminosity_base, tint, light_mode.x != 0u ? 0.30 : 0.36);
  blurred.a = 1.0;
  return blurred;
}

float4 ps_main(VertexOut input) : SV_Target {
  if (input.kind != 45 && magnifier_flags.y != 0u && magnifier_box.z > 0.0) {
    float2 half_size = magnifier_box.zw * 0.5;
    float2 local = input.position.xy - (magnifier_box.xy + half_size);
    if (rounded_distance(local, half_size,
                         max(magnifier_box.z / 24.0, 1.0)) <= 0.0)
      discard;
  }
  if (input.kind == 45) {
    // The lens is a quad drawn last instead of Metal's compute pass, so the
    // fragment ladder needs no cutout: nothing overdraws it.
    float2 box_size = max(magnifier_box.zw, 1.0);
    float2 local = input.position.xy - magnifier_box.xy;
    float2 half_size = box_size * 0.5;
    float distance = rounded_distance(local - half_size, half_size, 4.0);
    if (distance > 0.0) discard;
    float2 source_dimensions = max(magnifier_source.xy, 1.0);
    float2 source_center = magnifier_sample.xy * source_dimensions;
    float2 source_point = source_center + (local / box_size - 0.5) * 40.0;
    float2 sample_point = floor(source_point);
    float2 sample_uv = source_point / source_dimensions;
    bool in_source = all(sample_point >= 0.0) &&
        all(sample_point < source_dimensions) &&
        all(sample_uv >= magnifier_source_range.xy) &&
        all(sample_uv <= magnifier_source_range.zw);
    // Nearest neighbour keeps the magnified desktop pixels square.
    float4 pixel = in_source
        ? magnifier_texture.SampleLevel(
              point_sampler, (sample_point + 0.5) / source_dimensions, 0)
        : float4(0.15, 0.15, 0.16, 1.0);
    bool shade = ((magnifier_flags.x & 1u) != 0u && local.x < half_size.x) ||
                 ((magnifier_flags.x & 2u) != 0u && local.x >= half_size.x) ||
                 ((magnifier_flags.x & 4u) != 0u && local.y < half_size.y) ||
                 ((magnifier_flags.x & 8u) != 0u && local.y >= half_size.y);
    if (shade) {
      float3 shade_color = light_mode.x != 0u ? float3(0.0, 0.0, 0.0)
                                              : float3(1.0, 1.0, 1.0);
      pixel.rgb = lerp(pixel.rgb, shade_color, 0.1);
    }
    if (distance > -1.0) pixel = float4(0.15, 0.15, 0.16, 1.0);
    return pixel;
  }
  if (input.kind == 33) {
    // A frozen desktop is an opaque backing plane. Capture APIs may leave
    // alpha unspecified, but exposing it through a premultiplied composition
    // swap chain would reveal the live desktop beneath the snapshot.
    float4 sampled = snapshot.SampleLevel(linear_sampler, input.uv, 0);
    return float4(sampled.rgb, 1.0);
  }
  if (input.kind == 46) {
    // The rounded chrome plate. Kinds 12-14 stay deliberately rectangular
    // because on macOS the surface owned the radius; this one owns its own.
    float2 dimensions = 1.0 / max(fwidth(input.uv), float2(0.0001, 0.0001));
    float2 half_size = dimensions * 0.5;
    float radius = min(chrome.x, min(half_size.x, half_size.y));
    float distance =
        rounded_distance((input.uv - 0.5) * dimensions, half_size, radius);
    float aa = max(fwidth(distance), 0.0001);
    float coverage = clamp(0.5 - distance / aa, 0.0, 1.0);
    if (coverage <= 0.0) discard;
    float outline_mix = chrome_outline.a <= 0.002
        ? 0.0
        : clamp(0.5 + (distance + 1.0) / aa, 0.0, 1.0);
    float4 material = material_backdrop(input.position.xy);
    float3 material_tint = light_mode.x != 0u
        ? float3(0.92, 0.92, 0.93)
        : float3(0.035, 0.037, 0.042);
    material.rgb = lerp(
        material.rgb, material_tint, saturate(chrome.y) * 0.22);
    float4 control = action_fills[0];
    float4 color = float4(lerp(material.rgb, control.rgb, control.a), 1.0);
    color = lerp(color, chrome_outline, outline_mix * chrome_outline.a);
    color.a *= coverage;
    return color;
  }
  if (input.kind == 47) {
    // Chrome text. The texture carries white coverage and the tint comes from
    // the portable control foreground, which is what replaces the status
    // pill's AppKit text colour.
    float coverage = label.SampleLevel(linear_sampler, input.uv, 0).a;
    if (coverage <= 0.002) discard;
    float4 color = action_fills[1];
    color.a *= coverage;
    return color;
  }
  if (input.kind == 49 || input.kind == 50) {
    float coverage = input.kind == 50
        ? secondary_label.SampleLevel(linear_sampler, input.uv, 0).r
        : label.SampleLevel(linear_sampler, input.uv, 0).r;
    if (coverage <= 0.002) discard;
    float4 color = action_fills[1];
    color.a *= coverage;
    return color;
  }
  if (input.kind == 51) {
    float fill_coverage = label.SampleLevel(linear_sampler, input.uv, 0).r;
    float2 dimensions = 1.0 / max(fwidth(input.uv), float2(0.0001, 0.0001));
    float2 texel = 1.0 / dimensions;
    float ring = 0.0;
    [unroll]
    for (uint tap = 0; tap < 8; ++tap) {
      float angle = tap * 0.78539816;
      float2 direction = float2(cos(angle), sin(angle));
      ring += label.SampleLevel(
          linear_sampler, input.uv + direction * outlined_label.x * texel, 0).r;
    }
    float halo_coverage = max(fill_coverage, saturate(ring * 0.5));
    float3 fill = light_mode.x != 0u ? float3(0.149, 0.149, 0.149)
                                     : float3(1.0, 1.0, 1.0);
    float3 halo = light_mode.x != 0u ? float3(1.0, 1.0, 1.0)
                                     : float3(0.0, 0.0, 0.0);
    float halo_alpha = halo_coverage * (light_mode.x != 0u ? 1.0 : 0.8);
    float combined_alpha = fill_coverage + halo_alpha * (1.0 - fill_coverage);
    float3 premultiplied = fill * fill_coverage +
                           halo * halo_alpha * (1.0 - fill_coverage);
    return float4(combined_alpha > 0.0001
                      ? premultiplied / combined_alpha
                      : float3(0.0, 0.0, 0.0),
                  combined_alpha);
  }
  if (input.kind == 34 || input.kind == 35) {
    float2 dimensions = 1.0 / max(fwidth(input.uv), float2(0.0001, 0.0001));
    float width = input.kind == 34 ? max(ruler_animation.z, 1.0) : 3.0;
    float margin = width * 0.5 + 1.0;
    float2 half_size = dimensions * 0.5;
    float2 centerline_half = max(half_size - margin, float2(0.0, 0.0));
    float2 offset = abs((input.uv - 0.5) * dimensions) - centerline_half;
    float distance = length(max(offset, 0.0)) +
                     min(max(offset.x, offset.y), 0.0);
    float ring_distance = abs(distance) - width * 0.5;
    float aa = max(fwidth(distance), 0.5);
    float coverage = clamp(0.5 - ring_distance / aa, 0.0, 1.0);
    if (coverage <= 0.0) discard;
    float4 color = ruler_colors[0];
    color.a *= coverage * (input.kind == 34 ? ruler_animation.y : 0.32);
    return color;
  }
  if (input.kind >= 39 && input.kind <= 41) {
    float2 pixel_radius = 1.0 / max(fwidth(input.uv), float2(0.0001, 0.0001));
    float radius = (pixel_radius.x + pixel_radius.y) * 0.5;
    float2 local = input.uv * radius;
    float width = input.kind == 40 ? max(ruler_animation.z, 1.0) : 1.0;
    float half_width = width * 0.5;
    float radial = abs(length(local) - radius) - half_width;
    float quadrant = max(-local.x, -local.y);
    float arc_distance = max(radial, quadrant);
    float endpoint_x = length(local - float2(radius, 0.0)) - half_width;
    float endpoint_y = length(local - float2(0.0, radius)) - half_width;
    float distance = min(arc_distance, min(endpoint_x, endpoint_y));
    float aa = max(fwidth(distance), 0.5);
    float coverage = clamp(0.5 - distance / aa, 0.0, 1.0);
    if (input.kind == 41) {
      float along = atan2(max(local.y, 0.0), max(local.x, 0.0)) * radius;
      float phase = fmod(along, 7.0);
      float pattern_aa = max(fwidth(along), 0.5);
      float dash = 1.0 - smoothstep(4.0 - pattern_aa, 4.0 + pattern_aa, phase);
      coverage *= dash;
    }
    if (coverage <= 0.0) discard;
    float4 color = ruler_colors[0];
    color.a *= coverage * (input.kind == 40 ? ruler_animation.y
                                            : input.kind == 41 ? 0.7 : 1.0);
    return color;
  }
  if (input.kind == 37) {
    float4 sampled = secondary_label.SampleLevel(linear_sampler, input.uv, 0);
    if (sampled.a <= 0.002) discard;
    return float4(sampled.rgb / sampled.a, sampled.a * ruler_animation.w);
  }
  if (input.kind == 11 || input.kind == 15 || input.kind == 48) {
    float4 sampled = input.kind == 15
        ? secondary_label.SampleLevel(linear_sampler, input.uv, 0)
        : label.SampleLevel(linear_sampler, input.uv, 0);
    if (sampled.a <= 0.002) discard;
    float opacity = input.kind == 48 ? 1.0 - ruler_animation.w : 1.0;
    return float4(sampled.rgb / sampled.a, sampled.a * opacity);
  }
  if (input.kind == 28) return ruler_colors[0];
  if (input.kind >= 42 && input.kind <= 44) {
    float4 color = ruler_colors[0];
    color.a *= input.kind == 42 ? 0.45 : input.kind == 43 ? 0.85 : 0.30;
    return color;
  }
  if (input.kind == 36) return ruler_colors[1];
  if (input.kind == 38) {
    float4 color = ruler_colors[1];
    color.a *= ruler_animation.y;
    return color;
  }
  if (input.kind == 31) {
    float4 color = ruler_colors[0];
    color.a *= 0.32;
    return color;
  }
  if (input.kind == 32) {
    float4 color = ruler_colors[0];
    color.a *= ruler_animation.y;
    return color;
  }
  if (input.kind == 29) {
    float2 dimensions = 1.0 / max(fwidth(input.uv), float2(0.0001, 0.0001));
    float2 half_size = dimensions * 0.5;
    float radius = min(4.0, min(half_size.x, half_size.y));
    float distance =
        rounded_distance((input.uv - 0.5) * dimensions, half_size, radius);
    float aa = max(fwidth(distance), 0.0001);
    float coverage = clamp(0.5 - distance / aa, 0.0, 1.0);
    if (coverage <= 0.0) discard;
    float4 color = ruler_sample;
    color.a *= coverage * (1.0 - ruler_animation.x) * (1.0 - ruler_animation.w);
    return color;
  }
  if (input.kind == 30) {
    float4 color = action_fills[1];
    color.a *= ruler_animation.x * (1.0 - ruler_animation.w);
    return color;
  }
  if (input.kind >= 22 && input.kind <= 26) {
    float cell = float(input.kind - 21);
    float2 atlas_uv = float2((cell + input.uv.x) / 6.0, input.uv.y);
    // The source cells are 96px so a 14pt toolbar icon is a substantial
    // minification. A lone bilinear lookup covers only four source texels and
    // aliases the Lucide strokes; integrate a 4x4 footprint instead.
    float2 footprint_x = ddx(atlas_uv);
    float2 footprint_y = ddy(atlas_uv);
    float coverage = 0.0;
    [unroll]
    for (int y = 0; y < 4; y++) {
      [unroll]
      for (int x = 0; x < 4; x++) {
        float2 offset = footprint_x * ((float(x) + 0.5) / 4.0 - 0.5) +
                        footprint_y * ((float(y) + 0.5) / 4.0 - 0.5);
        coverage += icons.SampleLevel(linear_sampler, atlas_uv + offset, 0).r;
      }
    }
    coverage *= 1.0 / 16.0;
    if (coverage <= 0.002) discard;
    float4 color = action_fills[1];
    color.a *= coverage;
    return color;
  }
  if (input.kind >= 12 && input.kind <= 14) {
    // Material-backed OSC controls have one semantic radius owner: their
    // native surface. Keeping the fill rectangular prevents a second,
    // height-derived radius from diverging on composed controls such as the
    // two-row ruler loupe.
    return input.kind == 13 ? action_fills[1] : action_fills[0];
  }
  if (input.kind == 6) return overlay_shade;
  if (input.kind >= 17 && input.kind <= 20) {
    float2 dimensions = 1.0 / max(fwidth(input.uv), float2(0.0001, 0.0001));
    float2 half_size = dimensions * 0.5;
    float radius = min(2.0, min(half_size.x, half_size.y));
    float distance =
        rounded_distance((input.uv - 0.5) * dimensions, half_size, radius);
    float aa = max(fwidth(distance), 0.0001);
    float coverage = clamp(0.5 - distance / aa, 0.0, 1.0);
    if (coverage <= 0.0) discard;
    float4 fill = input.kind == 20 ? ocr_colors[6]
        : input.kind == 19 ? ocr_colors[4]
        : input.kind == 18 ? ocr_colors[2] : ocr_colors[0];
    float4 outline = input.kind == 20 ? ocr_colors[7]
        : input.kind == 19 ? ocr_colors[5]
        : input.kind == 18 ? ocr_colors[3] : ocr_colors[1];
    float outline_width = input.kind == 17 || input.kind == 18 ? 2.0 : 1.0;
    float outline_mix = clamp(0.5 + (distance + outline_width) / aa, 0.0, 1.0);
    float4 color = lerp(fill, outline, outline_mix);
    color.a *= coverage;
    return color;
  }
  if (input.kind >= 7 && input.kind <= 10) {
    bool horizontal = input.kind <= 8;
    float longitudinal = horizontal ? input.uv.x : input.uv.y;
    float transverse = horizontal ? input.uv.y : input.uv.x;
    float pixels_per_pattern_unit = 1.0 / max(fwidth(longitudinal), 0.0001);
    float thickness = 1.0 / max(fwidth(transverse), 0.0001);
    float edge_start = input.aux.x;
    float edge_end = edge_start + input.aux.y;
    float pattern_position = edge_start + longitudinal;
    float cycle_start = floor(pattern_position / 12.0) * 12.0;
    float segment_start = max(cycle_start, edge_start);
    float segment_end = min(cycle_start + 9.0, edge_end);
    if (segment_end <= segment_start) discard;
    float local_start = (segment_start - edge_start) * pixels_per_pattern_unit;
    float local_end = (segment_end - edge_start) * pixels_per_pattern_unit;
    float local_position = longitudinal * pixels_per_pattern_unit;
    float radius = thickness * 0.5;
    float cap_radius = min(radius, (local_end - local_start) * 0.5);
    float center_start = local_start + cap_radius;
    float center_end = local_end - cap_radius;
    float2 offset = float2(
        local_position - clamp(local_position, center_start, center_end),
        (transverse - 0.5) * thickness);
    float distance = length(offset) - cap_radius;
    float aa = max(fwidth(distance), 0.0001);
    float coverage = clamp(0.5 - distance / aa, 0.0, 1.0);
    if (coverage <= 0.0) discard;
    // A single outer capsule owns both colors. Keeping anti-aliasing at the
    // exterior prevents the fill from compositing over the one-pixel ring.
    float outline_mix = clamp(0.5 + (distance + 1.0) / aa, 0.0, 1.0);
    float4 color = lerp(control_colors[0], control_colors[1], outline_mix);
    color.a *= coverage;
    return color;
  }
  if (input.kind == 3 || input.kind == 16) {
    float2 dimensions = 1.0 / max(fwidth(input.uv), float2(0.0001, 0.0001));
    float2 offset = (input.uv - 0.5) * dimensions;
    float radius = max(min(dimensions.x, dimensions.y) * 0.5 - 1.0, 0.0);
    if (input.kind == 16) {
      bool horizontal = dimensions.x >= dimensions.y;
      float half_segment = abs(dimensions.x - dimensions.y) * 0.5;
      if (horizontal)
        offset.x -= clamp(offset.x, -half_segment, half_segment);
      else
        offset.y -= clamp(offset.y, -half_segment, half_segment);
    }
    float distance = length(offset) - radius;
    float aa = max(fwidth(distance), 0.0001);
    float coverage = clamp(0.5 - distance / aa, 0.0, 1.0);
    if (coverage <= 0.0) discard;
    float outline_mix = clamp(0.5 + (distance + 1.0) / aa, 0.0, 1.0);
    float4 color = lerp(control_colors[0], control_colors[1], outline_mix);
    color.a *= coverage;
    return color;
  }
  float coverage = 1.0;
  bool guide = input.kind == 4 || input.kind == 5;
  if (!guide && (input.kind & 1u) != 0u) {
    float2 dimensions = 1.0 / max(fwidth(input.uv), float2(0.0001, 0.0001));
    float2 offset = (input.uv - 0.5) * dimensions;
    float radius = max(min(dimensions.x, dimensions.y) * 0.5 - 1.0, 0.0);
    float edge = length(offset) - radius;
    float aa = max(fwidth(edge), 0.5);
    coverage = 1.0 - smoothstep(-aa, aa, edge);
    if (coverage <= 0.0) discard;
  }
  if (guide) {
    if (input.kind == 5)
      return light_mode.x != 0u ? float4(0.008, 0.518, 0.780, 1.0)
                                : float4(0.055, 0.647, 0.914, 1.0);
    return float4(0.918, 0.702, 0.031, 1.0);
  }
  bool halo = input.kind >= 2;
  float4 color = halo ? control_colors[1] : control_colors[0];
  color.a *= coverage;
  return color;
}
