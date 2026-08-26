// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

cbuffer Selection : register(b0) {
  float4 frame;       // viewport-local x/y/width/height in physical pixels
  float4 viewport;    // physical width/height, theme (0 dark, 1 light), visible
  float4 radius_control; // center x/y, visible, reserved
  float4 guides; // x, y, x-is-object, y-is-object (negative x/y means hidden)
  float4 crop_image; // image x/y/width/height; negative width disables crop shade
  float4 magnifier_box; // x/y/width/height; zero width disables the cutout
  float4 label; // size readout x/y/width/height in physical pixels; zero width hides it
  float4 secondary_label; // second intrinsic OSC action label, or zero when absent
  float4 label_params; // halo radius in pixels, display scale (pixels per point), reserved
  float4 action_shades; // primary light/dark, secondary light/dark
};

// Grayscale glyph coverage of the "W x H" readout, one texel per physical
// pixel of `label`, rasterised by GDI on the CPU (see `LabelTexture`).
Texture2D label_coverage : register(t0);
Texture2D secondary_label_coverage : register(t1);
SamplerState label_sampler : register(s0);

struct VertexOut { float4 position : SV_Position; };
VertexOut vs_main(uint id : SV_VertexID) {
  VertexOut output;
  float2 p = float2((id << 1) & 2, id & 2);
  output.position = float4(p * float2(2, -2) + float2(-1, 1), 0, 1);
  return output;
}

float circle_coverage(float2 pixel, float2 center, float radius) {
  // Like the Metal handle quads: fully opaque up to the radius, with the AA
  // ramp spent outside the edge, so the fill keeps its colour to the rim and
  // the ring around it stays one pixel wide.
  float distance = length(pixel - center) - radius;
  return 1.0 - smoothstep(0.0, 1.0, distance);
}

float line_coverage(float value, float edge, float half_width) {
  // Hard-edged like the Metal quads: full coverage inside the half width, a
  // one-pixel falloff outside, so a 1px core lands on exactly one pixel row.
  return 1.0 - smoothstep(half_width - 0.5, half_width + 0.5, abs(value - edge));
}

float rounded_distance(float2 pixel, float4 rect, float radius) {
  float2 half_size = rect.zw * 0.5;
  float2 local = abs(pixel - (rect.xy + half_size)) - (half_size - radius);
  return length(max(local, 0.0)) + min(max(local.x, local.y), 0.0) - radius;
}

// `color` is kept straight while the composition swap chain requires a
// premultiplied result. Updating both values as source-over here means curved
// coverage is multiplied exactly once when ps_main writes color * alpha.
void composite_layer(inout float3 color, inout float alpha,
                     float3 layer_color, float layer_alpha) {
  float combined_alpha = layer_alpha + alpha * (1.0 - layer_alpha);
  float3 premultiplied = layer_color * layer_alpha +
                         color * alpha * (1.0 - layer_alpha);
  color = combined_alpha > 0.0001 ? premultiplied / combined_alpha : 0.0;
  alpha = combined_alpha;
}

float label_sample(float2 uv) {
  return label_coverage.SampleLevel(label_sampler, uv, 0).r;
}

float secondary_label_sample(float2 uv) {
  return secondary_label_coverage.SampleLevel(label_sampler, uv, 0).r;
}

float4 ps_main(VertexOut input) : SV_Target {
  if (viewport.w < 0.5) return 0;
  float2 p = input.position.xy;
  if (magnifier_box.z > 0.0 &&
      rounded_distance(p, magnifier_box, max(magnifier_box.z / 24.0, 1.0)) <= 0.0)
    return 0;
  // Frame edges arrive on integer pixel boundaries while SV_Position samples
  // pixel centres; snap the lines onto the inner pixel row so the core is a
  // solid line rather than two half-covered grey ones.
  float left = frame.x + 0.5, top = frame.y + 0.5;
  float right = frame.x + frame.z - 0.5, bottom = frame.y + frame.w - 0.5;
  // Metal sizes the OSC in points: a 1pt core inside a 3pt halo, 4pt handles
  // with a one-device-pixel ring. Scale converts those to physical pixels.
  float scale = max(label_params.y, 0.1);
  float core_half = 0.5 * scale, halo_half = 1.5 * scale;
  float within_x = step(left - halo_half - 1.5, p.x) * step(p.x, right + halo_half + 1.5);
  float within_y = step(top - halo_half - 1.5, p.y) * step(p.y, bottom + halo_half + 1.5);
  float border = max(max(line_coverage(p.x, left, halo_half), line_coverage(p.x, right, halo_half)) * within_y,
                     max(line_coverage(p.y, top, halo_half), line_coverage(p.y, bottom, halo_half)) * within_x);
  float core = max(max(line_coverage(p.x, left, core_half), line_coverage(p.x, right, core_half)) * within_y,
                   max(line_coverage(p.y, top, core_half), line_coverage(p.y, bottom, core_half)) * within_x);
  if (crop_image.z >= 0.0) {
    float vertical_distance = min(abs(p.x - left), abs(p.x - right));
    float horizontal_distance = min(abs(p.y - top), abs(p.y - bottom));
    float coordinate = vertical_distance < horizontal_distance ? p.y - top : p.x - left;
    float wave = abs(frac(coordinate / 10.0) - 0.5);
    float aa = max(fwidth(wave), 0.001);
    float dash = 1.0 - smoothstep(0.30, 0.30 + aa, wave);
    border *= dash;
    core *= dash;
  }
  float2 handles[8] = {
    float2(left, top), float2(floor((left + right) * 0.5) + 0.5, top), float2(right, top),
    float2(right, floor((top + bottom) * 0.5) + 0.5), float2(right, bottom),
    float2(floor((left + right) * 0.5) + 0.5, bottom), float2(left, bottom),
    float2(left, floor((top + bottom) * 0.5) + 0.5)
  };
  float handle_radius = 4.0 * scale;
  float handle_ring = handle_radius + 1.0;
  float handle_outline = 0.0, handle_fill = 0.0;
  [unroll] for (uint index = 0; index < 8; ++index) {
    handle_outline = max(handle_outline, circle_coverage(p, handles[index], handle_ring));
    handle_fill = max(handle_fill, circle_coverage(p, handles[index], handle_radius));
  }
  if (radius_control.z > 0.5) {
    handle_outline = max(handle_outline, circle_coverage(p, radius_control.xy, handle_ring));
    handle_fill = max(handle_fill, circle_coverage(p, radius_control.xy, handle_radius));
  }
  float dark_theme = 1.0 - viewport.z;
  float3 primary = lerp(float3(0.09, 0.09, 0.10), 1.0, dark_theme);
  float3 contrast = lerp(1.0, float3(0.09, 0.09, 0.10), dark_theme);
  float contrast_alpha = saturate(max(border, handle_outline));
  float primary_alpha = saturate(max(core, handle_fill));
  // The crop shade is the bottom layer, as on macOS, so the border, handles
  // and guides composite over it at full strength instead of being dimmed.
  float crop_shade = 0.0;
  if (crop_image.z >= 0.0) {
    float image_inside = step(crop_image.x, p.x) * step(p.x, crop_image.x + crop_image.z) *
                         step(crop_image.y, p.y) * step(p.y, crop_image.y + crop_image.w);
    float crop_inside = step(frame.x, p.x) * step(p.x, frame.x + frame.z) *
                        step(frame.y, p.y) * step(p.y, frame.y + frame.w);
    crop_shade = image_inside * (1.0 - crop_inside);
  }
  float shade_alpha = crop_shade * 0.4;
  float3 color = float3(0.0, 0.0, 0.0);
  float alpha = shade_alpha;
  color = lerp(color, contrast, contrast_alpha);
  alpha = max(alpha, contrast_alpha);
  color = lerp(color, primary, primary_alpha);
  alpha = max(alpha, primary_alpha);
  float guide_x = guides.x >= 0.0 ? line_coverage(p.x, guides.x, 0.5) : 0.0;
  float guide_y = guides.y >= 0.0 ? line_coverage(p.y, guides.y, 0.5) : 0.0;
  float guide_alpha = max(guide_x, guide_y);
  float3 canvas_guide = dark_theme > 0.5 ? float3(0.98, 0.75, 0.12) : float3(0.78, 0.46, 0.02);
  float3 object_guide = dark_theme > 0.5 ? float3(0.18, 0.70, 0.95) : float3(0.00, 0.42, 0.70);
  float3 guide_color = guide_x > 0.0 ? (guides.z > 0.5 ? object_guide : canvas_guide)
                                    : (guides.w > 0.5 ? object_guide : canvas_guide);
  color = lerp(color, guide_color, guide_alpha);
  alpha = max(alpha, guide_alpha);
  // The size readout sits on top of everything. It is the Metal backend's
  // stroked-then-filled monospaced label: the fill is the glyph coverage, the
  // halo a dilation of that coverage by the stroke radius, so the text stays
  // legible over any pane content without a backing plate.
  if (label_params.z > 0.5 && label.z > 0.0) {
    // The label bitmap has 2pt horizontal inset and a 16pt text-xs line box.
    // These extents complete React's px-2/py-1 compact Button geometry.
    float4 primary_button = float4(label.xy - float2(6.0, 4.0) * scale,
                                   label.zw + float2(12.0, 8.0) * scale);
    float primary_coverage = 1.0 - smoothstep(
        -1.0, 1.0, rounded_distance(p, primary_button, 6.0 * scale));
    float secondary_coverage = 0.0;
    if (secondary_label.z > 0.0) {
      float4 secondary_button = float4(
          secondary_label.xy - float2(6.0, 4.0) * scale,
          secondary_label.zw + float2(12.0, 8.0) * scale);
      secondary_coverage = 1.0 - smoothstep(
          -1.0, 1.0, rounded_distance(p, secondary_button, 6.0 * scale));
    }
    // React resolves its neutral-soft and pressed semantic tokens to opaque
    // colours. These are the same sRGB mixes, while hover is neutral-100.
    float primary_shade = lerp(action_shades.x, action_shades.y, dark_theme);
    float secondary_shade = lerp(action_shades.z, action_shades.w, dark_theme);
    composite_layer(color, alpha, primary_shade.xxx, primary_coverage);
    composite_layer(color, alpha, secondary_shade.xxx, secondary_coverage);
  }
  if (label.z > 0.0 && p.x >= label.x && p.x <= label.x + label.z &&
      p.y >= label.y && p.y <= label.y + label.w) {
    float2 uv = (p - label.xy) / label.zw;
    float2 texel = 1.0 / label.zw;
    float fill_coverage = label_sample(uv);
    // A stroke tapers: the halo is the mean coverage of a ring of taps at the
    // stroke radius, stretched so a fully covered neighbourhood saturates and
    // a lone edge fades, rather than a hard `max` dilation.
    float halo_radius = label_params.x;
    float ring = 0.0;
    [unroll] for (uint tap = 0; tap < 8; ++tap) {
      float angle = tap * 0.78539816;
      float2 direction = float2(cos(angle), sin(angle));
      ring += label_sample(uv + direction * halo_radius * texel);
    }
    float halo_coverage = max(fill_coverage, saturate(ring * 0.5));
    float3 label_fill = lerp(float3(0.149, 0.149, 0.149), 1.0, dark_theme);
    float3 label_halo = lerp(1.0, 0.0, dark_theme);
    float halo_alpha = label_params.z > 0.5
      ? 0.0
      : saturate(halo_coverage) * lerp(1.0, 0.8, dark_theme);
    composite_layer(color, alpha, label_halo, halo_alpha);
    composite_layer(color, alpha, label_fill, saturate(fill_coverage));
  }
  if (secondary_label.z > 0.0 &&
      p.x >= secondary_label.x && p.x <= secondary_label.x + secondary_label.z &&
      p.y >= secondary_label.y && p.y <= secondary_label.y + secondary_label.w) {
    float2 uv = (p - secondary_label.xy) / secondary_label.zw;
    float coverage = secondary_label_sample(uv);
    float3 fill = lerp(float3(0.149, 0.149, 0.149), 1.0, dark_theme);
    composite_layer(color, alpha, fill, saturate(coverage));
  }
  return float4(color * alpha, alpha);
}
