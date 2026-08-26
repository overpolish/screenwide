// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#define SCREENWIDE_KEYBOARD_SHADER_SOURCE @R"METAL(
struct KeyboardKeyUniforms {
  uint x, width, visible, slot;
  float alpha, scale, progress, layout_progress;
  uint layout_from_mask, layout_to_mask;
};
struct KeyboardUniforms {
  uint width, height, key_count, animation;
  float scale, layout_progress, maximum_width, requested_scale;
  float center_x, center_y; KeyboardKeyUniforms keys[8];
};
static_assert(sizeof(KeyboardUniforms) == 360, "Keyboard uniforms must match their native layout");

static float keyboard_effective_scale(
    constant KeyboardUniforms &keyboard, float2 canvas_dimensions) {
  float requested = keyboard.requested_scale > 0.0
      ? keyboard.requested_scale : keyboard.scale;
  if (!(keyboard.maximum_width > 0.0) || any(canvas_dimensions <= 0.0))
    return requested;
  const float design_height = 20.0;
  const float edge_margin = 0.055;
  const float animation_extent = 1.12;
  float available_width = canvas_dimensions.x * (1.0 - edge_margin * 2.0);
  float width_at_unit_scale = canvas_dimensions.y * (60.0 / 1080.0) *
      keyboard.maximum_width / design_height;
  float fitted = available_width /
      max(width_at_unit_scale * animation_extent, 0.0001);
  return min(requested, fitted);
}

static float4 keyboard_texel(
    const device uchar4 *pixels, constant KeyboardUniforms &keyboard,
    float2 source) {
  source = clamp(source, 0.0, float2(keyboard.width - 1u,
                                    keyboard.height - 1u));
  uint2 low = uint2(floor(source));
  uint2 high = min(low + 1u, uint2(keyboard.width - 1u,
                                  keyboard.height - 1u));
  float2 fraction = fract(source);
  float4 a = float4(pixels[low.y * keyboard.width + low.x]) / 255.0;
  float4 b = float4(pixels[low.y * keyboard.width + high.x]) / 255.0;
  float4 c = float4(pixels[high.y * keyboard.width + low.x]) / 255.0;
  float4 d = float4(pixels[high.y * keyboard.width + high.x]) / 255.0;
  return mix(mix(a, b, fraction.x), mix(c, d, fraction.x), fraction.y);
}

static float4 keyboard_key_pixel(
    const device uchar4 *pixels, constant KeyboardUniforms &keyboard,
    constant KeyboardKeyUniforms &key, float2 point,
    float2 canvas_dimensions, float animation_scale, float x_offset) {
  if (animation_scale <= 0.0001) return float4(0.0);
  float height = canvas_dimensions.y * (60.0 / 1080.0) *
                 keyboard_effective_scale(keyboard, canvas_dimensions);
  float width = height * float(keyboard.width) / max(float(keyboard.height), 1.0);
  float bottom = canvas_dimensions.y * 0.055;
  float center_x = keyboard.center_x >= 0.0 ? keyboard.center_x * canvas_dimensions.x
                                           : canvas_dimensions.x * 0.5;
  float center_y = keyboard.center_y >= 0.0 ? keyboard.center_y * canvas_dimensions.y
                                           : canvas_dimensions.y - bottom - height * 0.5;
  float row_x = center_x - width * 0.5;
  float key_x = row_x + width * float(key.x) / float(keyboard.width);
  float key_width = width * float(key.width) / float(keyboard.width);
  float2 key_size = float2(key_width, height) * animation_scale;
  float2 center = float2(key_x + key_width * 0.5 + x_offset, center_y);
  float2 uv = (point - (center - key_size * 0.5)) / key_size;
  if (any(uv < 0.0) || any(uv > 1.0)) return float4(0.0);
  // `keyboard_texel` treats integer coordinates as texel centres. Convert
  // from rectangle-edge UVs accordingly and clamp to this key so linear
  // sampling cannot pull colour from its neighbouring gap or key.
  float2 source = float2(float(key.x) + uv.x * float(key.width) - 0.5,
                         uv.y * float(keyboard.height) - 0.5);
  source.x = clamp(source.x, float(key.x),
                   float(key.x + max(key.width, 1u) - 1u));
  source.y = clamp(source.y, 0.0, float(keyboard.height - 1u));
  return keyboard_texel(pixels, keyboard, source);
}

static float keyboard_motion_spring(float progress) {
  float t = clamp(progress, 0.0, 1.0);
  float phase = 6.0 * t;
  return t >= 1.0 ? 1.0
      : 1.0 - exp(-5.0 * t) * (cos(phase) + (5.0 / 6.0) * sin(phase));
}

static float keyboard_pop_spring(float progress) {
  return keyboard_motion_spring(progress);
}

static uint keyboard_slot_count(constant KeyboardUniforms &keyboard) {
  uint count = min(keyboard.key_count, 8u);
  uint slots = 0u;
  for (uint index = 0; index < count; ++index)
    slots = max(slots, keyboard.keys[index].slot + 1u);
  return slots;
}

static float keyboard_gap(constant KeyboardUniforms &keyboard) {
  uint count = min(keyboard.key_count, 8u);
  float gap = INFINITY;
  for (uint index = 1u; index < count; ++index) {
    float candidate = float(keyboard.keys[index].x) -
                      float(keyboard.keys[index - 1u].x +
                            keyboard.keys[index - 1u].width);
    if (candidate > 0.0) gap = min(gap, candidate);
  }
  return isfinite(gap) ? gap : 0.0;
}

static float keyboard_slot_width(
    constant KeyboardUniforms &keyboard, uint slot) {
  uint count = min(keyboard.key_count, 8u);
  float width = 0.0;
  for (uint index = 0; index < count; ++index)
    if (keyboard.keys[index].slot == slot)
      width = max(width, float(keyboard.keys[index].width));
  return width;
}

static float keyboard_slot_left(
    constant KeyboardUniforms &keyboard, uint slot, uint mask) {
  uint slots = keyboard_slot_count(keyboard);
  float gap = keyboard_gap(keyboard);
  uint included = popcount(mask);
  float total = gap * float(max(int(included) - 1, 0));
  for (uint candidate = 0; candidate < slots; ++candidate)
    if ((mask & (1u << candidate)) != 0u)
      total += keyboard_slot_width(keyboard, candidate);
  float left = (float(keyboard.width) - total) * 0.5;
  for (uint candidate = 0; candidate < slot; ++candidate)
    if ((mask & (1u << candidate)) != 0u)
      left += keyboard_slot_width(keyboard, candidate) + gap;
  return left;
}

static float keyboard_layout_offset(
    constant KeyboardUniforms &keyboard, uint index,
    float2 canvas_dimensions, float progress_delta) {
  uint count = min(keyboard.key_count, 8u);
  if (count < 2u) return 0.0;
  float height = canvas_dimensions.y * (60.0 / 1080.0) *
                 keyboard_effective_scale(keyboard, canvas_dimensions);
  float full_width = height * float(keyboard.width) /
                     max(float(keyboard.height), 1.0);
  constant KeyboardKeyUniforms &key = keyboard.keys[index];
  float slot_width = keyboard_slot_width(keyboard, key.slot);
  float from_center = keyboard_slot_left(
      keyboard, key.slot, key.layout_from_mask) + slot_width * 0.5;
  float to_center = keyboard_slot_left(
      keyboard, key.slot, key.layout_to_mask) + slot_width * 0.5;
  float progress = keyboard_motion_spring(
      max(key.layout_progress - progress_delta, 0.0));
  float target_center = mix(from_center, to_center, progress);
  float source_offset = target_center -
                        (float(key.x) + float(key.width) * 0.5);
  return source_offset * full_width / max(float(keyboard.width), 1.0);
}

static float4 composite_keyboard(
    float4 rgba, const device uchar4 *pixels,
    constant KeyboardUniforms &keyboard, float2 point, float2 dimensions) {
  if (keyboard.key_count == 0 || keyboard.width == 0 || keyboard.height == 0)
    return rgba;
  for (uint index = 0; index < min(keyboard.key_count, 8u); ++index) {
    constant KeyboardKeyUniforms &key = keyboard.keys[index];
    if (key.visible == 0 || key.alpha <= 0.0) continue;
    float4 value = float4(0.0);
    float total = 0.0;
    float layout_offset = keyboard_layout_offset(
        keyboard, index, dimensions, 0.0);
    float previous_offset = keyboard_layout_offset(
        keyboard, index, dimensions, 0.12);
    float layout_delta = previous_offset - layout_offset;
    bool pop_blur = keyboard.animation == 0u && key.progress < 1.0;
    bool layout_blur = abs(layout_delta) > 0.25;
    float requested_scale = keyboard.requested_scale > 0.0
        ? keyboard.requested_scale : keyboard.scale;
    float current_scale = key.scale / max(requested_scale, 0.001);
    float previous_progress = key.visible == 2u
        ? min(key.progress + 0.08, 1.0)
        : max(key.progress - 0.08, 0.0);
    float previous_scale = pop_blur
        ? keyboard_pop_spring(previous_progress)
        : current_scale;
    float key_height = dimensions.y * (60.0 / 1080.0) *
                       keyboard_effective_scale(keyboard, dimensions);
    float key_width = key_height * float(key.width) /
                      max(float(keyboard.height), 1.0);
    float radial_travel = 0.5 * length(float2(key_width, key_height)) *
                          abs(previous_scale - current_scale);
    float travel = max(abs(layout_delta), radial_travel);
    uint samples = (pop_blur || layout_blur)
        ? min(max(uint(ceil(travel / 0.75)) + 1u, 8u), 48u)
        : 1u;
    for (uint sample = 0; sample < samples; ++sample) {
      float amount = samples == 1u ? 0.0 : float(sample) / float(samples - 1u);
      float weight = exp(-2.5 * amount * amount);
      value += keyboard_key_pixel(pixels, keyboard, key, point, dimensions,
          mix(current_scale, previous_scale, amount),
          layout_offset + layout_delta * amount) * weight;
      total += weight;
    }
    value /= max(total, 1.0);
    float opacity = clamp(key.alpha, 0.0, 1.0);
    rgba.rgb = value.rgb * opacity + rgba.rgb * (1.0 - value.a * opacity);
    rgba.a = value.a * opacity + rgba.a * (1.0 - value.a * opacity);
  }
  return rgba;
}

static float4 keyboard_overlay_pixel(
    const device uchar4 *pixels, constant KeyboardUniforms &keyboard,
    float2 point, float2 dimensions) {
  float4 premultiplied = composite_keyboard(
      float4(0.0), pixels, keyboard, point, dimensions);
  if (premultiplied.a <= 0.0001) return float4(0.0);
  return float4(premultiplied.rgb / premultiplied.a, premultiplied.a);
}

kernel void overlay_keyboard_luma(
    const device uchar4 *pixels [[buffer(0)]],
    constant KeyboardUniforms &keyboard [[buffer(1)]],
    constant int2 &origin [[buffer(2)]],
    texture2d<float, access::read_write> luma [[texture(0)]],
    uint2 gid [[thread_position_in_grid]]) {
  int2 output = origin + int2(gid);
  if (any(output < 0) || output.x >= int(luma.get_width()) ||
      output.y >= int(luma.get_height())) return;
  float2 dimensions(luma.get_width(), luma.get_height());
  float4 rgba = keyboard_overlay_pixel(
      pixels, keyboard, float2(output) + 0.5, dimensions);
  if (rgba.a <= 0.0001) return;
  float value = 16.0 / 255.0 +
      dot(rgba.rgb, float3(0.182586, 0.614231, 0.062007));
  float existing = luma.read(uint2(output)).r;
  luma.write(mix(existing, value, rgba.a), uint2(output));
}

kernel void overlay_keyboard_chroma(
    const device uchar4 *pixels [[buffer(0)]],
    constant KeyboardUniforms &keyboard [[buffer(1)]],
    constant int2 &origin [[buffer(2)]],
    constant uint2 &output_dimensions [[buffer(3)]],
    texture2d<float, access::read_write> chroma [[texture(0)]],
    uint2 gid [[thread_position_in_grid]]) {
  int2 output_pixel = origin + int2(gid * 2u);
  int2 output = output_pixel / 2;
  if (any(output < 0) || output.x >= int(chroma.get_width()) ||
      output.y >= int(chroma.get_height())) return;
  float3 rgb_sum = 0.0;
  float alpha_sum = 0.0;
  for (uint y = 0; y < 2; ++y) {
    for (uint x = 0; x < 2; ++x) {
      float4 rgba = keyboard_overlay_pixel(
          pixels, keyboard, float2(output_pixel + int2(x, y)) + 0.5,
          float2(output_dimensions));
      rgb_sum += rgba.rgb * rgba.a;
      alpha_sum += rgba.a;
    }
  }
  float alpha = alpha_sum * 0.25;
  if (alpha <= 0.0001) return;
  float3 rgb = rgb_sum / max(alpha_sum, 0.0001);
  float2 value = float2(
      0.5 + dot(rgb, float3(-0.100644, -0.338572, 0.439216)),
      0.5 + dot(rgb, float3(0.439216, -0.398942, -0.040274)));
  float2 existing = chroma.read(uint2(output)).rg;
  chroma.write(float4(mix(existing, value, alpha), 0.0, 1.0),
               uint2(output));
}
)METAL"
