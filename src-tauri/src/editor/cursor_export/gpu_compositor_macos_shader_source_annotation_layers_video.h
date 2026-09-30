// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

/// The cursor and every annotation over an exported frame's two planes.
/// Highlights and spotlights change what is under them rather than drawing
/// over it, so the pass reads both planes before it writes either: one thread
/// takes a colour sample and the four luma pixels it covers, reads all of
/// them, draws the layer over each, and writes them back. The conversions
/// are the export's own.
///
/// `redo` is set for the pass run again after the screen layer is redrawn
/// over a camera sent behind it: the shade then goes only where the redraw
/// repainted, since everywhere else already has it. `source_luma` and
/// `source_chroma` are the screen frame the canvas was drawn from, after its
/// redactions, which a magnifier enlarges. `cursor_images` and
/// `cursor_uniforms` are the screen layer's cursor, zeroed on the camera's.
#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_LAYERS_VIDEO @R"METAL(
/// A canvas drawn from a video frame's two planes.
struct AnnotationMagnifyVideo {
  AnnotationMagnifyPlacement at;
  texture2d<float, access::read> luma;
  texture2d<float, access::read> chroma;
};

static float4 annotation_magnify_fetch(AnnotationMagnifyVideo tap, int2 texel) {
  uint2 at = uint2(texel);
  uint2 half_at = min(at / 2u, uint2(tap.chroma.get_width(), tap.chroma.get_height()) - 1u);
  return float4(yuv_to_rgb(tap.luma.read(at).r, tap.chroma.read(half_at).rg), 1.0);
}

kernel void annotation_layers_video(
    constant CanvasUniforms &canvas [[buffer(0)]],
    const device AnnotationUniforms *annotations [[buffer(12)]],
    constant uint &count [[buffer(13)]], constant uint &above [[buffer(14)]],
    const device AnnotationSample *samples [[buffer(15)]],
    const device uchar4 *numbers [[buffer(16)]],
    constant AnnotationTextAtlas &atlas [[buffer(17)]],
    const device packed_float2 *points [[buffer(18)]],
    constant uint &redo [[buffer(20)]],
    constant AnnotationCursorBlur &cursor_blur [[buffer(21)]],
    constant OverlayUniforms &cursor_uniforms [[buffer(22)]],
    texture2d<float, access::read_write> luma [[texture(0)]],
    texture2d<float, access::read_write> chroma [[texture(1)]],
    texture2d<float, access::read> source_luma [[texture(2)]],
    texture2d<float, access::read> source_chroma [[texture(3)]],
    texture2d_array<float, access::read> cursor_images [[texture(4)]],
    uint2 gid [[thread_position_in_grid]]) {
  if (gid.x >= chroma.get_width() || gid.y >= chroma.get_height()) return;
  AnnotationMagnifyVideo tap = {
      annotation_magnify_placement(
          canvas, uint2(source_luma.get_width(), source_luma.get_height())),
      source_luma, source_chroma};
  AnnotationCursorCanvas pointer = {
      {annotations, count, cursor_blur}, cursor_images, &cursor_uniforms, &canvas};
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
                                                 atlas, points, tap, pointer);
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
