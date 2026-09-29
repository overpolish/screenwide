// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

/// Every annotation over an exported frame's two planes, in document order.
/// Highlights and spotlights change what is under them rather than drawing
/// over it, so the pass reads both planes before it writes either: one thread
/// takes a colour sample and the four luma pixels it covers, reads all of
/// them, draws the annotations over each, and writes them back. The
/// conversions are the export's own.
///
/// `redo` is set for the pass run again after the screen layer is redrawn
/// over a camera sent behind it: the shade then goes only where the redraw
/// repainted, since everywhere else already has it.
#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_LAYERS_VIDEO @R"METAL(
kernel void annotation_layers_video(
    constant CanvasUniforms &canvas [[buffer(0)]],
    const device AnnotationUniforms *annotations [[buffer(12)]],
    constant uint &count [[buffer(13)]], constant uint &above [[buffer(14)]],
    const device AnnotationSample *samples [[buffer(15)]],
    const device uchar4 *numbers [[buffer(16)]],
    constant AnnotationTextAtlas &atlas [[buffer(17)]],
    const device packed_float2 *points [[buffer(18)]],
    constant uint &redo [[buffer(20)]],
    texture2d<float, access::read_write> luma [[texture(0)]],
    texture2d<float, access::read_write> chroma [[texture(1)]],
    uint2 gid [[thread_position_in_grid]]) {
  if (gid.x >= chroma.get_width() || gid.y >= chroma.get_height()) return;
  float2 sample_uv = chroma.read(gid).rg;
  float3 sum = float3(0.0);
  float pixels = 0.0;
  bool changed = false;
  for (uint y = 0u; y < 2u; ++y)
    for (uint x = 0u; x < 2u; ++x) {
      uint2 pixel = gid * 2u + uint2(x, y);
      if (pixel.x >= luma.get_width() || pixel.y >= luma.get_height()) continue;
      float2 point = float2(pixel) + 0.5;
      float4 base = float4(yuv_to_rgb(luma.read(pixel).r, sample_uv), 1.0);
      float gate = redo != 0u ? canvas_foreground_coverage(point, canvas) : 1.0;
      float4 drawn = composite_annotation_layers(base, annotations, count, above, point,
                                                 float2(1.0), 1.0, gate, samples, numbers,
                                                 atlas, points);
      sum += drawn.rgb;
      pixels += 1.0;
      if (any(abs(drawn.rgb - base.rgb) > 1e-4)) {
        changed = true;
        luma.write(float4(16.0 / 255.0 + dot(saturate(drawn.rgb),
                                               float3(0.182586, 0.614231, 0.062007))),
                   pixel);
      }
    }
  if (!changed || pixels <= 0.0) return;
  float3 rgb = saturate(sum / pixels);
  float2 value = float2(0.5 + dot(rgb, float3(-0.100644, -0.338572, 0.439216)),
                        0.5 + dot(rgb, float3(0.439216, -0.398942, -0.040274)));
  chroma.write(float4(value, 0.0, 1.0), gid);
}
)METAL"
