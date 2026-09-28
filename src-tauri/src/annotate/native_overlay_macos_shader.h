// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

/// The live overlay's one kernel.
///
/// Everything that decides what an annotation looks like - the curve solve, the head
/// geometry, the coverage and its feathering - is the editor's own
/// `composite_annotations`, compiled into this library from the same source
/// the export uses. This kernel only chooses the surface: a transparent
/// target, one annotation list, no camera layer.
///
/// A highlight recolours what is under it, which the overlay never sees: it
/// reads `underlay` instead, the desktop captured when it was drawn, laid over
/// the display's pixels.
///
/// Annotations arrive in this display's layer pixels, so the canvas placement is the
/// identity and `source_dimensions` is one pixel per pixel.
#define SCREENWIDE_ANNOTATE_SHADER_SOURCE @R"METAL(
kernel void annotate_overlay(
    constant CanvasUniforms &canvas [[buffer(0)]],
    const device AnnotationUniforms *annotations [[buffer(12)]],
    constant uint &count [[buffer(13)]],
    constant uint &above_camera [[buffer(14)]],
    const device AnnotationSample *samples [[buffer(15)]],
    const device uchar4 *numbers [[buffer(16)]],
    constant AnnotationTextAtlas &atlas [[buffer(17)]],
    const device packed_float2 *points [[buffer(18)]],
    texture2d<float, access::write> target [[texture(0)]],
    texture2d<float, access::sample> underlay [[texture(1)]],
    uint2 gid [[thread_position_in_grid]]) {
  if (gid.x >= target.get_width() || gid.y >= target.get_height()) return;
  constexpr sampler desktop(filter::linear, address::clamp_to_edge);
  float2 size = float2(target.get_width(), target.get_height());
  float4 base = underlay.sample(desktop, (float2(gid) + 0.5) / size);
  // Blended over nothing, so the result is already premultiplied - which is
  // what WindowServer composites a non-opaque layer with.
  float4 value = composite_highlights(float4(0), float4(base.rgb, 1.0), annotations, count,
      above_camera, float2(gid) + 0.5, 1.0, points);
  value = composite_annotations(value, annotations, count, above_camera,
      float2(gid) + 0.5, canvas, float2(1), 1.0, samples, numbers, atlas);
  target.write(value, gid);
}
)METAL"
