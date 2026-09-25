// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

/// The redaction passes over a video frame's two planes, before the canvas
/// samples them: the same cells, colours and cover as over an RGBA source,
/// converted the way the export converts everything it writes. A recording's
/// boxes are snapped to even pixels, so every colour sample lies wholly
/// inside a box or wholly outside it. The planes are the export's own copies
/// of the decoded frame, which the canvas then reads in their place.
#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_REDACT_VIDEO @R"METAL(
/// A video frame's planes, as the cells pass reads them.
struct RedactVideoPixels {
  texture2d<float, access::read> luma;
  texture2d<float, access::read> chroma;
  uchar4 at(uint2 point) const {
    float3 rgb = yuv_to_rgb(luma.read(point).r, chroma.read(point / 2u).rg);
    return uchar4(uchar3(round(saturate(rgb) * 255.0)), 255);
  }
};

/// The cells pass over a video frame: one threadgroup a cell.
kernel void redact_cells_video(
    texture2d<float, access::read> luma [[texture(0)]],
    texture2d<float, access::read> chroma [[texture(1)]],
    constant RedactUniforms &r [[buffer(1)]],
    device uint *cells [[buffer(3)]],
    uint index [[threadgroup_position_in_grid]],
    uint lane [[thread_index_in_threadgroup]]) {
  threadgroup uint sums[4][256];
  redact_average(RedactVideoPixels{luma, chroma}, r, cells, index, lane, sums);
}

/// The luma plane, one thread a pixel of the box.
kernel void redact_video_luma(
    texture2d<float, access::read_write> luma [[texture(0)]],
    constant RedactUniforms &r [[buffer(1)]],
    const device float2 *entries [[buffer(2)]],
    const device uint *cells [[buffer(3)]],
    uint2 gid [[thread_position_in_grid]]) {
  uint2 point = uint2(r.x0, r.y0) + gid;
  if (point.x >= r.x1 || point.y >= r.y1) return;
  float cover = redact_share(r, gid);
  if (cover <= 0.0) return;
  float3 rgb = saturate(redact_colour(r, entries, cells, gid));
  float value = 16.0 / 255.0 + dot(rgb, float3(0.182586, 0.614231, 0.062007));
  luma.write(mix(luma.read(point).r, value, cover), point);
}

/// The chroma plane, one thread a colour sample of the box: the average of
/// its four pixels' colours, each as much as it is covered.
kernel void redact_video_chroma(
    texture2d<float, access::read_write> chroma [[texture(0)]],
    constant RedactUniforms &r [[buffer(1)]],
    const device float2 *entries [[buffer(2)]],
    const device uint *cells [[buffer(3)]],
    uint2 gid [[thread_position_in_grid]]) {
  uint2 sample = uint2(r.x0, r.y0) / 2u + gid;
  if (sample.x * 2u >= r.x1 || sample.y * 2u >= r.y1) return;
  float3 sum = float3(0.0);
  float covered = 0.0;
  for (uint y = 0u; y < 2u; ++y)
    for (uint x = 0u; x < 2u; ++x) {
      uint2 point = sample * 2u + uint2(x, y);
      if (point.x < r.x0 || point.y < r.y0 || point.x >= r.x1 || point.y >= r.y1) continue;
      uint2 local = point - uint2(r.x0, r.y0);
      float cover = redact_share(r, local);
      if (cover <= 0.0) continue;
      sum += saturate(redact_colour(r, entries, cells, local)) * cover;
      covered += cover;
    }
  if (covered <= 0.0) return;
  float3 rgb = sum / covered;
  float2 value = float2(0.5 + dot(rgb, float3(-0.100644, -0.338572, 0.439216)),
                        0.5 + dot(rgb, float3(0.439216, -0.398942, -0.040274)));
  chroma.write(float4(mix(chroma.read(sample).rg, value, covered * 0.25), 0.0, 1.0), sample);
}
)METAL"
