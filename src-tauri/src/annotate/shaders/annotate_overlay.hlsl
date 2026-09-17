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
// Annotations arrive in this display's layer pixels, so there is no canvas
// placement to apply and one drawn pixel is one annotation pixel.

#include "../../editor/preview_platform/surface_windows/shaders/annotations.hlsl"

cbuffer Overlay : register(b0) {
  /// How many prepared arrows `annotation_arrows` holds.
  uint annotation_count;
  /// How wide an edge is smoothed, in layer pixels.
  float annotation_feather;
  float2 padding;
};

float4 vs_main(uint id : SV_VertexID) : SV_Position {
  float2 position = float2((id << 1) & 2, id & 2);
  return float4(position * float2(2, -2) + float2(-1, 1), 0, 1);
}

float4 ps_main(float4 position : SV_Position) : SV_Target {
  // Composed over nothing, so the result is already premultiplied - which is
  // what DirectComposition expects of a premultiplied swap chain.
  // The live overlay draws no counters yet, so it rasterises no numbers and
  // passes an empty atlas: the number pass returns before it samples.
  return composite_annotations(
      float4(0, 0, 0, 0), position.xy, 0u, annotation_count, annotation_feather,
      uint2(0, 0));
}
