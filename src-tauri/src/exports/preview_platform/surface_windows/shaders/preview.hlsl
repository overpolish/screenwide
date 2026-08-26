// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

cbuffer Canvas : register(b0) {
  float4 output_source; // output width/height, source width/height
  float4 image_rect;
  float4 crop_rect, source_crop_rect;
  float4 solid_color;
  float4 base_color;
  float4 recenter_inset_color;
  float4 mesh_points[8];
  float4 mesh_colors[4];
  float4 effects; // image radius, background radius, warp, shadow sigma
  float4 motion; // timeline seconds
  float4 cursor_geometry; // source-space anchor x/y, artwork width/height
  float4 cursor_effects; // reserved x/y, rotation radians, scale
  float4 cursor_blur; // source-space frame delta x/y
  float4 camera_frame; // output-space x/y/width/height
  float4 camera_crop; // camera source-space x/y/width/height
  float4 camera_effects; // radius, enabled, shadow sigma, camera on top
  float4 magnifier, magnifier_options, magnifier_bounds;
  float4 native_cursor_hotspots[8]; // normalized atlas hotspot x/y
  uint4 options; // seed, mesh enabled, point count, shadow enabled
  uint4 cursor_options; // artwork, enabled, clip to video, foreground only
};
Texture2D source_image : register(t0);
Texture2DArray native_cursor_images : register(t1);
Texture2D camera_image : register(t2);
Texture2D keyboard_image : register(t3);
// Mirrors the Metal `KeyboardUniforms` struct. HLSL pads struct array elements
// to 16 bytes, so the per-key fields live in parallel uint4/float4 arrays whose
// packing is identical on both backends.
cbuffer Keyboard : register(b1) {
  uint4 keyboard_dimensions; // artwork width/height, key count, animation
  float4 keyboard_animation; // scale, layout progress, maximum width, requested scale
  uint4 keyboard_key_geometry[8]; // artwork x, artwork width, visible, slot
  float4 keyboard_key_motion[8]; // alpha, scale, progress, layout progress
  uint4 keyboard_key_masks[8]; // layout from mask, layout to mask
};
SamplerState linear_sampler : register(s0);
SamplerState point_sampler : register(s1);
float hash(float2 position, uint seed) {
  float value = sin(dot(position, float2(127.1, 311.7)) + (float)seed * 0.017) * 43758.5453;
  return frac(value) * 2.0 - 1.0;
}

float noise(float2 position, uint seed) {
  float2 cell = floor(position), local = frac(position);
  float2 eased = local * local * (3.0 - 2.0 * local);
  float top = lerp(hash(cell, seed), hash(cell + float2(1, 0), seed), eased.x);
  float bottom = lerp(hash(cell + float2(0, 1), seed), hash(cell + 1, seed), eased.x);
  return lerp(top, bottom, eased.y);
}
float fractal_noise(float2 position, uint seed) {
  return noise(position, seed) * 0.58
    + noise(position * 2.07 + float2(11.3, -4.9), seed ^ 0x68bc21eb) * 0.28
    + noise(position * 4.19 + float2(-8.7, 13.1), seed ^ 0x02e5be93) * 0.14;
}
float rounded_distance(float2 pixel, float4 rect, float radius) {
  float2 half_size = rect.zw * 0.5;
  float2 local = abs(pixel - (rect.xy + half_size)) - (half_size - radius);
  return length(max(local, 0.0)) + min(max(local.x, local.y), 0.0) - radius;
}
float rounded_coverage(float2 pixel, float4 rect, float radius) {
  float distance = rounded_distance(pixel, rect, radius);
  if (radius <= 0.0) return distance < 0.0 ? 1.0 : 0.0;
  return 1.0 - smoothstep(-0.75, 0.75, distance);
}
float visible_shadow(float2 pixel, float sigma) {
  float2 shadow_pixel = pixel - float2(0, sigma * 0.35);
  // The foreground is the intersection of the crop window and placed source.
  // A tall crop around a wide source must not cast a tall rectangular shadow.
  float crop_distance = rounded_distance(shadow_pixel, crop_rect, effects.x);
  float image_distance = rounded_distance(shadow_pixel, image_rect, 0.0);
  float distance = max(recenter_inset_color.a > 0.0 ? crop_distance : max(crop_distance, image_distance), 0.0);
  return (36.0 / 255.0) * exp(-0.5 * distance * distance / (sigma * sigma));
}
float4 cursor_sample(float2 source_pixel, float2 anchor) {
  // The D3D preview's screen-space rotation convention is opposite to the
  // raster/Metal convention used to calculate the motion lean.
  float angle = -cursor_effects.z;
  if (cursor_options.x == 2) angle += 1.57079632679;
  float2 delta = source_pixel - anchor;
  float cosine = cos(angle), sine = sin(angle);
  float2 local = float2(cosine * delta.x + sine * delta.y,
                        -sine * delta.x + cosine * delta.y) / max(cursor_effects.w, 0.01);
  float2 atlas_uv = local / cursor_geometry.zw + native_cursor_hotspots[cursor_options.x].xy;
  if (any(atlas_uv < 0.0) || any(atlas_uv >= 1.0)) return 0.0;
  return native_cursor_images.Sample(linear_sampler, float3(atlas_uv, (float)cursor_options.x));
}
float4 cursor_layer(float2 pixel) {
  if (cursor_options.y == 0) return 0.0;
  float2 source_pixel = (pixel - image_rect.xy) / image_rect.zw * output_source.zw;
  // The recorded hotspot is source-space, but the atlas is sampled in the
  // pane's physical output. Snap that transformed hotspot to a pixel edge so
  // pixel-centred samples hit the cursor artwork consistently instead of
  // bilinearly splitting a thin I-beam or crosshair across two pixels.
  float2 display_anchor = image_rect.xy + cursor_geometry.xy / output_source.zw * image_rect.zw;
  float2 anchor = (round(display_anchor) - image_rect.xy) / image_rect.zw * output_source.zw;
  float travel = min(length(cursor_blur.xy), 80.0);
  float radius = length(cursor_geometry.zw) * cursor_effects.w + travel + 4.0;
  if (any(abs(source_pixel - anchor) > radius)) return 0.0;
  if (travel <= 1.25) return cursor_sample(source_pixel, anchor);
  float2 direction = cursor_blur.xy / max(length(cursor_blur.xy), 0.001);
  float4 accumulated = 0.0;
  float total_weight = 0.0;
  [unroll] for (uint index = 0; index < 24; ++index) {
    float progress = (float)index / 23.0;
    float centered = (progress - 0.5) / 0.34;
    float weight = exp(-0.5 * centered * centered);
    float2 sample_anchor = anchor + direction * ((progress - 0.8) * travel);
    float4 sample = cursor_sample(source_pixel, sample_anchor);
    accumulated.rgb += sample.rgb * sample.a * weight;
    accumulated.a += sample.a * weight;
    total_weight += weight;
  }
  accumulated.a /= total_weight;
  accumulated.rgb = accumulated.a > 0.0 ? accumulated.rgb / (total_weight * accumulated.a) : 0.0;
  return accumulated;
}
float3 background(float2 pixel) {
  if (options.y == 0) return solid_color.rgb;
  float shortest = min(output_source.x, output_source.y);
  float2 dimensions = output_source.xy;
  float2 aspect = dimensions / shortest;
  float frequency = 3.5 / shortest;
  float phase = motion.x * 0.28;
  float2 drift = float2(sin(phase), cos(phase * 0.83)) * shortest * 0.012;
  float2 warped_pixel = pixel + drift;
  float warp_scale = shortest * effects.z / 100.0;
  float2 warp = float2(
    fractal_noise(warped_pixel * frequency + phase * 0.035, options.x),
    fractal_noise(warped_pixel * frequency + float2(19.7, -7.3) - phase * 0.03, options.x ^ 0xa511e9b3)
  ) * warp_scale;
  float3 weighted = base_color.rgb * 0.18;
  float total = 0.18;
  [loop] for (uint index = 0; index < options.z; ++index) {
    float4 first = mesh_points[index * 2];
    float4 second = mesh_points[index * 2 + 1];
    float local_phase = phase + (float)index * 1.73;
    float2 animated_center = first.xy + float2(sin(local_phase), cos(local_phase * 0.91)) * 0.012;
    float2 delta = (pixel + warp) / shortest - animated_center * aspect;
    float2 rotated = float2(delta.x * second.x + delta.y * second.y,
                            -delta.x * second.y + delta.y * second.x);
    float distance = length(rotated / max(first.zw, 0.01));
    float weight = 1.0 / (pow(max(distance, 0.025), 3.5) + 0.012);
    weighted += mesh_colors[index].rgb * weight;
    total += weight;
  }
  float3 result = weighted / total;
  float depth = fractal_noise((pixel + drift) * frequency * 0.7, options.x ^ 0xd1b54a35) * 13.0 / 255.0;
  return saturate(result + depth);
}
float4 vs_main(uint id : SV_VertexID) : SV_Position {
  float2 position = float2((id << 1) & 2, id & 2);
  return float4(position * float2(2, -2) + float2(-1, 1), 0, 1);
}
float4 camera_layer(float4 result, float2 pixel) {
  if (camera_effects.y == 0.0) return result;
  float camera_alpha = rounded_coverage(pixel, camera_frame, camera_effects.x);
  if (camera_effects.z > 1.0) {
    float2 shadow_pixel = pixel - float2(0, camera_effects.z * 0.35);
    float distance = max(rounded_distance(
      shadow_pixel, camera_frame, camera_effects.x), 0.0);
    float shadow = (36.0 / 255.0) * exp(
      -0.5 * distance * distance / (camera_effects.z * camera_effects.z));
    result.rgb *= 1.0 - shadow * (1.0 - camera_alpha);
  }
  float2 camera_local = (pixel - camera_frame.xy) / camera_frame.zw;
  if (camera_alpha > 0.0 && all(camera_local >= 0.0) && all(camera_local <= 1.0)) {
    float2 camera_source_pixel = camera_crop.xy + camera_local * camera_crop.zw;
    uint camera_width, camera_height;
    camera_image.GetDimensions(camera_width, camera_height);
    float2 camera_uv = camera_source_pixel / float2(camera_width, camera_height);
    float4 camera = camera_image.Sample(linear_sampler, camera_uv);
    result.rgb = lerp(result.rgb, camera.rgb, camera.a * camera_alpha);
    result.a = camera.a * camera_alpha + result.a * (1.0 - camera.a * camera_alpha);
  }
  return result;
}
uint keyboard_key_count() { return min(keyboard_dimensions.z, 8u); }
// Shader model 4 has no countbits intrinsic.
uint keyboard_popcount(uint mask) {
  uint count = 0;
  [loop] for (uint bit = 0; bit < 32; ++bit) count += (mask >> bit) & 1u;
  return count;
}
float keyboard_effective_scale(float2 canvas_dimensions) {
  float requested = keyboard_animation.w > 0.0 ? keyboard_animation.w : keyboard_animation.x;
  if (!(keyboard_animation.z > 0.0) || any(canvas_dimensions <= 0.0)) return requested;
  const float design_height = 20.0;
  const float edge_margin = 0.055;
  const float animation_extent = 1.12;
  float available_width = canvas_dimensions.x * (1.0 - edge_margin * 2.0);
  float width_at_unit_scale = canvas_dimensions.y * (60.0 / 1080.0) *
      keyboard_animation.z / design_height;
  float fitted = available_width / max(width_at_unit_scale * animation_extent, 0.0001);
  return min(requested, fitted);
}
float4 keyboard_texel(float2 source) {
  float2 last = float2(keyboard_dimensions.xy) - 1.0;
  source = clamp(source, 0.0, last);
  int2 low = int2(floor(source));
  int2 high = min(low + 1, int2(last));
  float2 fraction = frac(source);
  float4 a = keyboard_image.Load(int3(low.x, low.y, 0));
  float4 b = keyboard_image.Load(int3(high.x, low.y, 0));
  float4 c = keyboard_image.Load(int3(low.x, high.y, 0));
  float4 d = keyboard_image.Load(int3(high.x, high.y, 0));
  return lerp(lerp(a, b, fraction.x), lerp(c, d, fraction.x), fraction.y);
}
float4 keyboard_key_pixel(uint index, float2 canvas_point, float2 canvas_dimensions,
                          float animation_scale, float x_offset) {
  if (animation_scale <= 0.0001) return 0.0;
  uint4 geometry = keyboard_key_geometry[index];
  float height = canvas_dimensions.y * (60.0 / 1080.0) *
                 keyboard_effective_scale(canvas_dimensions);
  float width = height * float(keyboard_dimensions.x) /
                max(float(keyboard_dimensions.y), 1.0);
  float bottom = canvas_dimensions.y * 0.055;
  float row_x = (canvas_dimensions.x - width) * 0.5;
  float key_x = row_x + width * float(geometry.x) / float(keyboard_dimensions.x);
  float key_width = width * float(geometry.y) / float(keyboard_dimensions.x);
  float2 key_size = float2(key_width, height) * animation_scale;
  float2 center = float2(key_x + key_width * 0.5 + x_offset,
                         canvas_dimensions.y - bottom - height * 0.5);
  float2 uv = (canvas_point - (center - key_size * 0.5)) / key_size;
  if (any(uv < 0.0) || any(uv > 1.0)) return 0.0;
  // `keyboard_texel` treats integer coordinates as texel centres. Convert
  // from rectangle-edge UVs accordingly and clamp to this key so linear
  // sampling cannot pull colour from its neighbouring gap or key.
  float2 source = float2(float(geometry.x) + uv.x * float(geometry.y) - 0.5,
                         uv.y * float(keyboard_dimensions.y) - 0.5);
  source.x = clamp(source.x, float(geometry.x),
                   float(geometry.x + max(geometry.y, 1u) - 1u));
  source.y = clamp(source.y, 0.0, float(keyboard_dimensions.y - 1u));
  return keyboard_texel(source);
}
float keyboard_motion_spring(float progress) {
  float t = saturate(progress);
  float phase = 6.0 * t;
  return t >= 1.0 ? 1.0
      : 1.0 - exp(-5.0 * t) * (cos(phase) + (5.0 / 6.0) * sin(phase));
}
uint keyboard_slot_count() {
  uint count = keyboard_key_count();
  uint slots = 0;
  [loop] for (uint index = 0; index < count; ++index)
    slots = max(slots, keyboard_key_geometry[index].w + 1u);
  return slots;
}
float keyboard_gap() {
  uint count = keyboard_key_count();
  float gap = 1e30;
  [loop] for (uint index = 1; index < count; ++index) {
    float candidate = float(keyboard_key_geometry[index].x) -
        float(keyboard_key_geometry[index - 1].x + keyboard_key_geometry[index - 1].y);
    if (candidate > 0.0) gap = min(gap, candidate);
  }
  return gap < 1e30 ? gap : 0.0;
}
float keyboard_slot_width(uint slot) {
  uint count = keyboard_key_count();
  float width = 0.0;
  [loop] for (uint index = 0; index < count; ++index)
    if (keyboard_key_geometry[index].w == slot)
      width = max(width, float(keyboard_key_geometry[index].y));
  return width;
}
float keyboard_slot_left(uint slot, uint mask) {
  uint slots = keyboard_slot_count();
  float gap = keyboard_gap();
  uint included = keyboard_popcount(mask);
  float total = gap * float(max(int(included) - 1, 0));
  [loop] for (uint candidate = 0; candidate < slots; ++candidate)
    if ((mask & (1u << candidate)) != 0u) total += keyboard_slot_width(candidate);
  float left = (float(keyboard_dimensions.x) - total) * 0.5;
  [loop] for (uint walked = 0; walked < slot; ++walked)
    if ((mask & (1u << walked)) != 0u) left += keyboard_slot_width(walked) + gap;
  return left;
}
float keyboard_layout_offset(uint index, float2 canvas_dimensions, float progress_delta) {
  uint count = keyboard_key_count();
  if (count < 2u) return 0.0;
  float height = canvas_dimensions.y * (60.0 / 1080.0) *
                 keyboard_effective_scale(canvas_dimensions);
  float full_width = height * float(keyboard_dimensions.x) /
                     max(float(keyboard_dimensions.y), 1.0);
  uint4 geometry = keyboard_key_geometry[index];
  uint4 masks = keyboard_key_masks[index];
  float slot_width = keyboard_slot_width(geometry.w);
  float from_center = keyboard_slot_left(geometry.w, masks.x) + slot_width * 0.5;
  float to_center = keyboard_slot_left(geometry.w, masks.y) + slot_width * 0.5;
  float progress = keyboard_motion_spring(
      max(keyboard_key_motion[index].w - progress_delta, 0.0));
  float target_center = lerp(from_center, to_center, progress);
  float source_offset = target_center - (float(geometry.x) + float(geometry.y) * 0.5);
  return source_offset * full_width / max(float(keyboard_dimensions.x), 1.0);
}
float4 composite_keyboard(float4 rgba, float2 canvas_point, float2 dimensions) {
  if (keyboard_dimensions.z == 0 || keyboard_dimensions.x == 0 || keyboard_dimensions.y == 0)
    return rgba;
  uint count = keyboard_key_count();
  [loop] for (uint index = 0; index < count; ++index) {
    uint4 geometry = keyboard_key_geometry[index];
    float4 motion = keyboard_key_motion[index];
    if (geometry.z == 0 || motion.x <= 0.0) continue;
    float4 value = 0.0;
    float total = 0.0;
    float layout_offset = keyboard_layout_offset(index, dimensions, 0.0);
    float previous_offset = keyboard_layout_offset(index, dimensions, 0.12);
    float layout_delta = previous_offset - layout_offset;
    bool pop_blur = keyboard_dimensions.w == 0u && motion.z < 1.0;
    bool layout_blur = abs(layout_delta) > 0.25;
    float requested_scale = keyboard_animation.w > 0.0
        ? keyboard_animation.w : keyboard_animation.x;
    float current_scale = motion.y / max(requested_scale, 0.001);
    float previous_progress = geometry.z == 2u
        ? min(motion.z + 0.08, 1.0) : max(motion.z - 0.08, 0.0);
    float previous_scale = pop_blur
        ? keyboard_motion_spring(previous_progress) : current_scale;
    float key_height = dimensions.y * (60.0 / 1080.0) *
                       keyboard_effective_scale(dimensions);
    float key_width = key_height * float(geometry.y) /
                      max(float(keyboard_dimensions.y), 1.0);
    float radial_travel = 0.5 * length(float2(key_width, key_height)) *
                          abs(previous_scale - current_scale);
    float travel = max(abs(layout_delta), radial_travel);
    uint samples = (pop_blur || layout_blur)
        ? min(max((uint)ceil(travel / 0.75) + 1u, 8u), 48u) : 1u;
    [loop] for (uint step_index = 0; step_index < samples; ++step_index) {
      float amount = samples == 1u ? 0.0 : float(step_index) / float(samples - 1u);
      float weight = exp(-2.5 * amount * amount);
      value += keyboard_key_pixel(index, canvas_point, dimensions,
          lerp(current_scale, previous_scale, amount),
          layout_offset + layout_delta * amount) * weight;
      total += weight;
    }
    value /= max(total, 1.0);
    float opacity = saturate(motion.x);
    rgba.rgb = value.rgb * opacity + rgba.rgb * (1.0 - value.a * opacity);
    rgba.a = value.a * opacity + rgba.a * (1.0 - value.a * opacity);
  }
  return rgba;
}
float4 ps_main(float4 position : SV_Position) : SV_Target {
  float2 pixel = position.xy;
  float background_alpha = rounded_coverage(pixel, float4(0, 0, output_source.xy), effects.y);
  float4 result = cursor_options.w != 0
    ? float4(0.0, 0.0, 0.0, 0.0)
    : float4(background(pixel), 1.0);
  if (camera_effects.w == 0.0) result = camera_layer(result, pixel);
  float crop_alpha = rounded_coverage(pixel, crop_rect, effects.x);
  // Axis-aligned zero-radius source edges are already pixel-exact. Smoothing
  // them leaks a fractional row of canvas colour around a default crop.
  float image_rect_alpha = rounded_coverage(pixel, image_rect, 0.0);
  float image_alpha = crop_alpha * image_rect_alpha * rounded_coverage(pixel, source_crop_rect, 0.0);
  float frame_alpha = recenter_inset_color.a > 0.0 ? crop_alpha : image_alpha;
  if (options.w != 0 && effects.w > 1.0) {
    float shadow = visible_shadow(pixel, effects.w);
    if (cursor_options.w != 0) {
      result.a = shadow * (1.0 - frame_alpha);
    } else {
      result.rgb *= 1.0 - shadow * (1.0 - frame_alpha);
    }
  }
  if (recenter_inset_color.a > 0.0)
    result = lerp(result, float4(recenter_inset_color.rgb, 1.0), crop_alpha);
  float2 uv = (pixel - image_rect.xy) / image_rect.zw;
  if (image_alpha > 0.0) {
    float4 video = source_image.Sample(linear_sampler, uv);
    result = lerp(result, float4(video.rgb, 1.0), video.a * image_alpha);
  }
  float4 cursor = cursor_layer(pixel);
  if (cursor_options.z != 0) cursor.a *= image_alpha;
  result.rgb = lerp(result.rgb, cursor.rgb, cursor.a);
  result.a = cursor.a + result.a * (1.0 - cursor.a);
  if (camera_effects.w != 0.0) result = camera_layer(result, pixel);
  result = composite_keyboard(result, pixel, output_source.xy);
  if (cursor_options.w == 0) {
    result.rgb = saturate(result.rgb + hash(pixel, 0x9e3779b9) / 255.0);
  }
  if (magnifier.z > 0.0) {
    float2 center = magnifier.xy;
    float2 box_size = magnifier.zz;
    float2 box_origin = center - box_size * 0.5;
    float corner_radius = max(magnifier.z / 24.0, 1.0);
    float distance = rounded_distance(pixel, float4(box_origin, box_size), corner_radius);
    if (distance <= 0.0) {
      bool use_camera = magnifier_options.x != 0.0;
      float2 source_pixel;
      float2 dimensions;
      if (use_camera) {
        float2 local = (center - camera_frame.xy) / camera_frame.zw;
        source_pixel = camera_crop.xy + local * camera_crop.zw + (pixel - center) / max(magnifier.w, 1.0);
        uint width, height;
        camera_image.GetDimensions(width, height);
        dimensions = float2(width, height);
        result = all(source_pixel >= 0.0) && all(source_pixel < dimensions) ? camera_image.Sample(point_sampler, source_pixel / dimensions) : float4(0.15, 0.15, 0.16, 1.0);
      } else {
        float2 local = (center - image_rect.xy) / image_rect.zw;
        source_pixel = local * output_source.zw + (pixel - center) / max(magnifier.w, 1.0);
        dimensions = output_source.zw;
        float2 source_uv = source_pixel / dimensions;
        bool in_effective_source = all(source_uv >= magnifier_bounds.xy) &&
          all(source_uv <= magnifier_bounds.xy + magnifier_bounds.zw);
        result = all(source_pixel >= 0.0) && all(source_pixel < dimensions) && in_effective_source
          ? source_image.Sample(point_sampler, source_uv) : float4(0.15, 0.15, 0.16, 1.0);
      }
      uint edges = (uint)magnifier_options.y;
      bool shade = ((edges & 1u) != 0u && pixel.x < center.x)
        || ((edges & 2u) != 0u && pixel.x >= center.x)
        || ((edges & 4u) != 0u && pixel.y < center.y)
        || ((edges & 8u) != 0u && pixel.y >= center.y);
      if (shade) {
        float3 shade_color = magnifier_options.z != 0.0
          ? float3(0.0, 0.0, 0.0) : float3(1.0, 1.0, 1.0);
        result.rgb = lerp(result.rgb, shade_color, 0.1);
      }
      float border_coverage = smoothstep(-1.5, -0.5, distance);
      result.rgb = lerp(result.rgb, float3(0.15, 0.15, 0.16), border_coverage);
      result.a = 1.0;
    }
  }
  // DirectComposition consumes premultiplied alpha. Clip every composed layer
  // to the rounded canvas and premultiply only once at the final boundary.
  result *= background_alpha;
  return result;
}
