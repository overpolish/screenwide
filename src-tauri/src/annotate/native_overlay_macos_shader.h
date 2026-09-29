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
/// the display's pixels. A spotlight's shade is black laid over the desktop,
/// and its blur is `softened`, the same desktop softened once in Rust, laid
/// under the shade where the blur reaches. A 1x1 `softened` is none at all.
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
    texture2d<float, access::sample> softened [[texture(2)]],
    uint2 gid [[thread_position_in_grid]]) {
  if (gid.x >= target.get_width() || gid.y >= target.get_height()) return;
  constexpr sampler desktop(filter::linear, address::clamp_to_edge);
  float2 size = float2(target.get_width(), target.get_height());
  float2 point = float2(gid) + 0.5;
  float2 across = point / size;
  float4 base = underlay.sample(desktop, across);
  // Blended over nothing, so the result is already premultiplied - which is
  // what WindowServer composites a non-opaque layer with. The blur is the
  // softened desktop laid over the live one as far as it reaches, and the
  // shade darkens both: what shows through is the desktop times what the
  // blur leaves of it times what the shade leaves.
  float4 value = float4(0.0);
  if (softened.get_width() > 1u) {
    float blur = annotation_spotlight_cover(annotations, count, above_camera, point, 1.0, true);
    if (blur > 0.0) value = float4(softened.sample(desktop, across).rgb * blur, blur);
  }
  float lift = 1.0 - annotation_spotlight_dim *
      annotation_spotlight_cover(annotations, count, above_camera, point, 1.0, false);
  value = float4(value.rgb * lift, 1.0 - (1.0 - value.a) * lift);
  value = composite_highlights(value, float4(base.rgb, 1.0), annotations, count, above_camera,
                               point, 1.0, points, samples);
  value = composite_annotations(value, annotations, count, above_camera, point, canvas,
                                float2(1), 1.0, samples, numbers, atlas);
  target.write(value, gid);
}
)METAL"
