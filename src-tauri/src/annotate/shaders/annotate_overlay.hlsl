// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The live overlay's one pass, the twin of `native_overlay_macos_shader.h`.
//
// Everything that decides what an annotation looks like - the curve solve, the
// head geometry, the coverage and its feathering - is the editor's own
// `composite_annotations`, included from the same source the still and the
// export compose through. This shader only chooses the surface: a transparent
// target, one annotation list, no camera layer.
//
// A highlight recolours what is under it, which the overlay never sees: it
// reads `annotate_underlay` instead, the desktop captured when it was drawn, or
// the still it is baked into. A spotlight's shade is black laid over the
// desktop, and its blur is `annotate_softened`, the same desktop softened once
// in Rust, laid under the shade where the blur reaches. A 1x1 softened texture
// is none at all.
//
// Annotations arrive in this display's layer pixels, so there is no canvas
// placement to apply and one drawn pixel is one annotation pixel.

#include "../../editor/preview_platform/surface_windows/shaders/annotations.hlsl"

cbuffer Overlay : register(b0) {
  /// How many prepared arrows `annotation_arrows` holds.
  uint annotation_count;
  /// How wide an edge is smoothed, in layer pixels.
  float annotation_feather;
  /// The size of the texture the counters' numbers were rasterised into, or
  /// zeroes on a frame with no counter to rasterise one for.
  uint2 annotation_number_atlas;
  /// How many atlas pixels that texture holds per layer pixel.
  float annotation_number_scale;
  /// The target's size in pixels, which the underlay is read across.
  float2 annotation_target;
  float annotation_spare;
};

/// What the highlights recolour, laid over the whole target.
Texture2D<float4> annotate_underlay : register(t10);
/// The same softened for the spotlights' blur, stretched over the target.
Texture2D<float4> annotate_softened : register(t11);

float4 vs_main(uint id : SV_VertexID) : SV_Position {
  float2 position = float2((id << 1) & 2, id & 2);
  return float4(position * float2(2, -2) + float2(-1, 1), 0, 1);
}

/// The softened desktop at `across`, a share of the target, read between its
/// four nearest pixels: it is a fraction of the target's size, and read
/// pixel by pixel it would show its blocks.
float3 annotate_softened_at(float2 across, uint2 size) {
  float2 texel = across * float2(size) - 0.5;
  float2 base = floor(texel);
  float2 t = texel - base;
  int2 last = int2(size) - 1;
  int2 low = clamp(int2(base), int2(0, 0), last);
  int2 high = clamp(int2(base) + 1, int2(0, 0), last);
  float3 top = lerp(annotate_softened.Load(int3(low.x, low.y, 0)).rgb,
                    annotate_softened.Load(int3(high.x, low.y, 0)).rgb, t.x);
  float3 bottom = lerp(annotate_softened.Load(int3(low.x, high.y, 0)).rgb,
                       annotate_softened.Load(int3(high.x, high.y, 0)).rgb, t.x);
  return lerp(top, bottom, t.y);
}

float4 ps_main(float4 position : SV_Position) : SV_Target {
  uint width, height;
  annotate_underlay.GetDimensions(width, height);
  float2 across = position.xy / max(annotation_target, float2(1.0, 1.0));
  uint2 texel = min(uint2(across * float2(width, height)), uint2(width - 1u, height - 1u));
  float4 base = float4(annotate_underlay.Load(int3(texel, 0)).rgb, 1.0);
  // Composed over nothing, so the result is already premultiplied - which is
  // what DirectComposition expects of a premultiplied swap chain. The blur is
  // the softened desktop laid over the live one as far as it reaches, and the
  // shade darkens both: what shows through is the desktop times what the blur
  // leaves of it times what the shade leaves.
  float4 result = float4(0, 0, 0, 0);
  uint soft_width, soft_height;
  annotate_softened.GetDimensions(soft_width, soft_height);
  if (soft_width > 1u) {
    float blur = annotation_spotlight_cover(position.xy, 0u, annotation_count,
                                            annotation_feather, true);
    if (blur > 0.0)
      result = float4(annotate_softened_at(across, uint2(soft_width, soft_height)) * blur, blur);
  }
  float lift = 1.0 - annotation_spotlight_dim *
      annotation_spotlight_cover(position.xy, 0u, annotation_count, annotation_feather, false);
  result = float4(result.rgb * lift, 1.0 - (1.0 - result.a) * lift);
  result = composite_highlights(result, base, position.xy, 0u, annotation_count,
                                annotation_feather);
  AnnotationTextAtlas atlas = {annotation_number_atlas, annotation_number_scale};
  return composite_annotations(
      result, position.xy, 0u, annotation_count, annotation_feather, atlas);
}
