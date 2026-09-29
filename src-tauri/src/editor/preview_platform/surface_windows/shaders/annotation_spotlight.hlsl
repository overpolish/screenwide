// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The shade a layer's spotlights cast. Every spotlight showing cuts its hole
// in one shared shade, which darkens what is under it by a fixed share. The
// HLSL twin of `gpu_compositor_macos_shader_source_annotation_spotlight.h`,
// and like it the arithmetic is Rust's `spotlight::geometry::{light, shade}`.
// Every number it reads was prepared by `spotlight::geometry::prepare_spotlight`.
// Included by `annotations.hlsl`, after the pieces every kind shares.

static const uint annotation_spotlight_kind = 6u;
// How much of the light the shade takes away. The twin of Rust's
// `spotlight::model::SPOTLIGHT_DIM`.
static const float annotation_spotlight_dim = 0.4;
// The flag a spotlight that blurs carries: `flags::BLUR`.
static const uint annotation_spotlight_blur = 1u << 3;

// How much of one spotlight's light reaches `probe`: all of it well inside
// the box, none outside it, and a smooth fall over its softness in from the
// edge. `pixel` is one drawn pixel, which is all a spotlight with no softness
// fades over.
float annotation_spotlight_light(PreviewGeometry shape, float2 probe, float pixel) {
  float2 low = float2(shape.ax, shape.ay);
  float2 high = float2(shape.bx, shape.by);
  float rounding = min(shape.rounding, min(high.x - low.x, high.y - low.y) * 0.5);
  float2 q = abs(probe - (low + high) * 0.5) - ((high - low) * 0.5 - rounding);
  float distance = length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - rounding;
  float soft = max(shape.width, 0.0);
  float fall = saturate((distance + soft + pixel * 0.5) / (soft + pixel));
  return 1.0 - fall * fall * (3.0 - 2.0 * fall);
}

// How much of a spotlight layer reaches `probe`, from the spotlights between
// `first` and `last`: as strong as the most present spotlight that counts -
// every one for the shade, those that blur for the blur - lifted by each
// one's light as far as it is present. A spotlight's presence rides in its
// colour's alpha, which the preparation scales by its reveal. `feather` is
// half a drawn pixel, as every annotation layer is given it. The editor's
// passes blur the source with the redactions; only the live overlay, which
// has no source, asks for the blur here.
float annotation_spotlight_cover(float2 probe, uint first, uint last, float feather,
                                 bool blurring) {
  float pixel = max(2.0 * feather, 1e-4);
  float layer = 0.0, lit = 0.0;
  for (uint index = first; index < last; ++index) {
    PreviewArrow annotation = annotation_arrows[index];
    if (annotation.kind != annotation_spotlight_kind) continue;
    float presence = saturate(annotation.alpha);
    if (presence <= 0.0) continue;
    if (!blurring || (annotation.flags & annotation_spotlight_blur) != 0u)
      layer = max(layer, presence);
    PreviewGeometry shape = annotation.geometry;
    // Only a point inside the box can be lit; everywhere else is shade.
    float reach = pixel + 1.0;
    if (probe.x < shape.ax - reach || probe.y < shape.ay - reach ||
        probe.x > shape.bx + reach || probe.y > shape.by + reach)
      continue;
    lit = max(lit, presence * annotation_spotlight_light(shape, probe, pixel));
  }
  return max(layer - lit, 0.0);
}

// `rgba`, premultiplied, under the shade the spotlights between `first` and
// `last` cast at `canvas_point`.
float4 composite_spotlights(float4 rgba, float2 canvas_point, uint first, uint last,
                            float feather) {
  float shade = annotation_spotlight_cover(canvas_point, first, last, feather, false);
  rgba.rgb *= 1.0 - annotation_spotlight_dim * shade;
  return rgba;
}
