// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

Texture2D<float4> source_texture : register(t0);
SamplerState linear_sampler : register(s0);

cbuffer Piece : register(b0) {
  uint2 output_size;
  uint2 source_size;
  uint2 source_origin;
  uint2 source_extent;
  uint2 destination_origin;
  uint2 destination_extent;
};

struct VertexOutput {
  float4 position : SV_Position;
  float2 uv : TEXCOORD0;
};

VertexOutput vs_main(uint vertex_id : SV_VertexID) {
  static const float2 corners[6] = {
    float2(0.0, 0.0), float2(1.0, 0.0), float2(1.0, 1.0),
    float2(0.0, 0.0), float2(1.0, 1.0), float2(0.0, 1.0)
  };
  float2 corner = corners[vertex_id];
  float2 pixel = float2(destination_origin) + corner * float2(destination_extent);
  VertexOutput output;
  output.position = float4(
    pixel.x / float(output_size.x) * 2.0 - 1.0,
    1.0 - pixel.y / float(output_size.y) * 2.0,
    0.0,
    1.0
  );
  output.uv =
    (float2(source_origin) + corner * float2(source_extent)) / float2(source_size);
  return output;
}

float4 ps_main(VertexOutput input) : SV_Target {
  return source_texture.Sample(linear_sampler, input.uv);
}
