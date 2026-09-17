// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#ifndef SCREENWIDE_OSC_GPU_MACOS_SHADER_TYPES_H
#define SCREENWIDE_OSC_GPU_MACOS_SHADER_TYPES_H

/// The vertex layout, the palettes the fragment stage is handed, and the
/// pass-through vertex stage.
#define SCREENWIDE_REGION_OSC_SHADER_TYPES @R"METAL(
struct region_osc_vertex {
  float2 position;
  float2 uv;
  uint kind;
  uint padding;
};

struct region_osc_out {
  float4 position [[position]];
  float2 uv;
  uint kind;
};

struct region_osc_control_palette {
  float4 fill;
  float4 outline;
};

struct region_osc_action_palette {
  float4 primary;
  float4 secondary;
};

struct region_osc_ocr_palette {
  float4 primary_fill;
  float4 primary_outline;
  float4 qr_fill;
  float4 qr_outline;
  float4 error_fill;
  float4 error_outline;
  float4 selection_fill;
  float4 selection_outline;
};

struct region_osc_ruler_palette {
  float4 primary;
  float4 info;
};

vertex region_osc_out region_osc_vertex_main(
    const device region_osc_vertex *vertices [[buffer(0)]],
    uint index [[vertex_id]]) {
  region_osc_out out;
  out.position = float4(vertices[index].position, 0.0, 1.0);
  out.uv = vertices[index].uv;
  out.kind = vertices[index].kind;
  return out;
}
)METAL"

#endif
