// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The recording's audio ribbon: a row of rounded bars, one per four-point
// bucket of the level envelope, scrolling under the playhead.

struct Ribbon {
  color: vec4<f32>,
  flat_color: vec4<f32>,
  geometry: vec4<f32>, // viewport width/height, bar pitch/width
  style: vec4<f32>, // maximum half-height, scale, playhead ratio, envelope points
}

@group(0) @binding(0) var<uniform> ribbon: Ribbon;
@group(0) @binding(1) var levels: texture_2d<f32>;

// One triangle over the viewport.
@vertex
fn vs_main(@builtin(vertex_index) id: u32) -> @builtin(position) vec4<f32> {
  let corner = vec2<f32>(f32((id << 1u) & 2u), f32(id & 2u));
  return vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  let pixel = position.xy;
  // Taken before any branch: a derivative needs the whole quad.
  let pixel_width = max(fwidth(pixel.x), 1.0);
  let pitch = max(ribbon.geometry.z, 1.0);
  let radius = ribbon.geometry.w * 0.5;
  let count = ceil(ribbon.geometry.x / pitch) + 2.0;
  let middle = floor(count * 0.5);
  let centre_bucket = ribbon.style.z * max(ribbon.style.w - 1.0, 0.0) / 4.0;
  let origin = ribbon.geometry.x * 0.5 - middle * pitch - radius - fract(centre_bucket) * pitch;
  let column = round((pixel.x - origin - radius) / pitch);
  if (column < 0.0 || column >= count) {
    return vec4<f32>(0.0);
  }

  // Bars belong to fixed four-point buckets. Moving the row continuously
  // keeps their heights stable as the waveform passes under the playhead.
  let bucket = floor(centre_bucket) + column - middle;
  let outside = bucket < 0.0 || bucket >= ceil(ribbon.style.w / 4.0);
  var amplitude = 0.0;
  if (!outside) {
    amplitude = saturate(textureLoad(levels, vec2<i32>(i32(bucket), 0), 0).r);
  }
  let half_height = max(ribbon.style.y, sqrt(amplitude) * ribbon.style.x);
  let centre_x = origin + column * pitch + radius;
  let straight = max(half_height - radius, 0.0);
  let vertical_distance = max(abs(pixel.y - ribbon.geometry.y * 0.5) - straight, 0.0);
  let cross_section = sqrt(max(radius * radius - vertical_distance * vertical_distance, 0.0));

  // Integrate the horizontal capsule slice over the pixel, matching Metal's
  // coverage. Fractional scrolling preserves brightness instead of flickering.
  let overlap = max(min(pixel.x + pixel_width * 0.5, centre_x + cross_section) -
                    max(pixel.x - pixel_width * 0.5, centre_x - cross_section), 0.0);
  let coverage = saturate(overlap / pixel_width);
  let selected = select(ribbon.color, ribbon.flat_color, outside);
  let alpha = selected.a * coverage;
  return vec4<f32>(selected.rgb * alpha, alpha);
}
