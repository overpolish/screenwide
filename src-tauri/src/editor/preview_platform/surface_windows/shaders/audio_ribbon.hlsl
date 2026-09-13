// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

cbuffer Ribbon : register(b0) {
  float4 color;
  float4 flat_color;
  float4 geometry; // viewport width/height, bar pitch/width
  float4 style; // maximum half-height, scale, playhead ratio, envelope points
};
Texture2D<float> levels : register(t0);

struct Vertex { float4 position : SV_POSITION; };
Vertex vs_main(uint id : SV_VertexID) {
  float2 corner = float2((id << 1) & 2, id & 2);
  Vertex output;
  output.position = float4(corner * 2.0 - 1.0, 0.0, 1.0);
  return output;
}

float4 ps_main(Vertex input) : SV_TARGET {
  float2 pixel = input.position.xy;
  float pitch = max(geometry.z, 1.0);
  float radius = geometry.w * 0.5;
  float count = ceil(geometry.x / pitch) + 2.0;
  float middle = floor(count * 0.5);
  float centre_bucket = style.z * max(style.w - 1.0, 0.0) / 4.0;
  float origin = geometry.x * 0.5 - middle * pitch - radius - frac(centre_bucket) * pitch;
  float column = round((pixel.x - origin - radius) / pitch);
  if (column < 0.0 || column >= count) return 0.0;

  // Bars belong to fixed four-point buckets. Moving the row continuously
  // keeps their heights stable as the waveform passes under the playhead.
  float bucket = floor(centre_bucket) + column - middle;
  bool outside = bucket < 0.0 || bucket >= ceil(style.w / 4.0);
  float amplitude = 0.0;
  if (!outside) amplitude = saturate(levels.Load(int3((int)bucket, 0, 0)));
  float half_height = max(style.y, sqrt(amplitude) * style.x);
  float centre_x = origin + column * pitch + radius;
  float straight = max(half_height - radius, 0.0);
  float vertical_distance = max(abs(pixel.y - geometry.y * 0.5) - straight, 0.0);
  float cross_section = sqrt(max(radius * radius - vertical_distance * vertical_distance, 0.0));

  // Integrate the horizontal capsule slice over the pixel, matching Metal's
  // coverage. Fractional scrolling preserves brightness instead of flickering.
  float pixel_width = max(fwidth(pixel.x), 1.0);
  float overlap = max(min(pixel.x + pixel_width * 0.5, centre_x + cross_section)
    - max(pixel.x - pixel_width * 0.5, centre_x - cross_section), 0.0);
  float coverage = saturate(overlap / pixel_width);
  float4 selected = outside ? flat_color : color;
  float alpha = selected.a * coverage;
  return float4(selected.rgb * alpha, alpha);
}
