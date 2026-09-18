// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#define GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_VIDEO @R"METAL(
static float4 annotation_video_pixel(const device AnnotationUniforms *annotations,
    uint count, uint above, float2 point, constant CanvasUniforms &canvas,
    const device AnnotationSample *samples, const device uchar4 *numbers,
    uint2 atlas) {
  float4 value = composite_annotations(float4(0), annotations, count, above, point,
      canvas, float2(1), 1.0, samples, numbers, atlas);
  return value.a > 0.0001 ? float4(value.rgb / value.a, value.a) : float4(0);
}
kernel void overlay_annotation_luma(
    constant CanvasUniforms &canvas [[buffer(0)]],
    const device AnnotationUniforms *annotations [[buffer(12)]],
    constant uint &count [[buffer(13)]], constant uint &above [[buffer(14)]],
    const device AnnotationSample *samples [[buffer(15)]],
    const device uchar4 *numbers [[buffer(16)]],
    constant uint2 &atlas [[buffer(17)]],
    texture2d<float, access::read_write> luma [[texture(0)]], uint2 gid [[thread_position_in_grid]]) {
  if (gid.x >= luma.get_width() || gid.y >= luma.get_height()) return;
  float4 rgba = annotation_video_pixel(annotations, count, above, float2(gid) + 0.5, canvas, samples, numbers, atlas);
  if (rgba.a <= 0.0001) return;
  float value = 16.0 / 255.0 + dot(rgba.rgb, float3(0.182586, 0.614231, 0.062007));
  luma.write(mix(luma.read(gid).r, value, rgba.a), gid);
}
kernel void overlay_annotation_chroma(
    constant CanvasUniforms &canvas [[buffer(0)]],
    const device AnnotationUniforms *annotations [[buffer(12)]],
    constant uint &count [[buffer(13)]], constant uint &above [[buffer(14)]],
    const device AnnotationSample *samples [[buffer(15)]],
    const device uchar4 *numbers [[buffer(16)]],
    constant uint2 &atlas [[buffer(17)]],
    texture2d<float, access::read_write> chroma [[texture(0)]], uint2 gid [[thread_position_in_grid]]) {
  if (gid.x >= chroma.get_width() || gid.y >= chroma.get_height()) return;
  float3 sum = 0; float alpha_sum = 0;
  for (uint y = 0; y < 2; y++) for (uint x = 0; x < 2; x++) {
    float4 rgba = annotation_video_pixel(annotations, count, above, float2(gid * 2u + uint2(x,y)) + 0.5, canvas, samples, numbers, atlas);
    sum += rgba.rgb * rgba.a; alpha_sum += rgba.a;
  }
  if (alpha_sum <= 0.0001) return;
  float3 rgb = sum / alpha_sum;
  float2 value = float2(0.5 + dot(rgb, float3(-0.100644, -0.338572, 0.439216)),
                       0.5 + dot(rgb, float3(0.439216, -0.398942, -0.040274)));
  chroma.write(float4(mix(chroma.read(gid).rg, value, alpha_sum * 0.25), 0, 1), gid);
}
)METAL"
