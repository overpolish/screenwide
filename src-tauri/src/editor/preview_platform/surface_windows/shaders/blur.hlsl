// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Separable Gaussian post-process for a suspended preview pane. WebView2's
// compositor cannot sample the DirectComposition swap chains beneath it, so
// the DOM's `backdrop-filter` blurs nothing here the way CoreAnimation blurs
// the Metal pane on macOS: the pane blurs its own presented pixels instead,
// matching the CSS blur the Windows chrome applies to itself.
cbuffer Blur : register(b0) {
  float4 blur_texel; // 1/width, 1/height, sigma in pane pixels, tap pairs
  float4 blur_axis; // sample direction x/y, tap spacing in pane pixels
};
Texture2D blur_source : register(t0);
SamplerState linear_sampler : register(s0);

float4 vs_main(uint id : SV_VertexID) : SV_Position {
  float2 position = float2((id << 1) & 2, id & 2);
  return float4(position * float2(2, -2) + float2(-1, 1), 0, 1);
}

float4 ps_main(float4 position : SV_Position) : SV_Target {
  float2 uv = position.xy * blur_texel.xy;
  float sigma = max(blur_texel.z, 0.0001);
  float falloff = -0.5 / (sigma * sigma);
  int pairs = (int)blur_texel.w;
  // Taps are spaced further apart than one pixel only when the kernel is
  // wider than the fixed tap budget: the pane is then composed well above its
  // on-screen size, so DirectComposition's linear downscale resolves the
  // sparser sampling away.
  float spacing = max(blur_axis.z, 1.0);
  // The pane composes with premultiplied alpha (its swap chain is
  // `DXGI_ALPHA_MODE_PREMULTIPLIED`), and premultiplied colour blurs
  // correctly under a plain weighted sum - no unpremultiply round trip.
  float4 total = blur_source.SampleLevel(linear_sampler, uv, 0);
  float total_weight = 1.0;
  [loop]
  for (int index = 1; index <= pairs; ++index) {
    // Two Gaussian taps per hardware sample: the linear filter interpolates
    // the pair, halving the sample count for the same kernel.
    float first = spacing * (float)(index * 2 - 1);
    float second = spacing * (float)(index * 2);
    float first_weight = exp(falloff * first * first);
    float second_weight = exp(falloff * second * second);
    float weight = first_weight + second_weight;
    float offset = (first * first_weight + second * second_weight) / max(weight, 1e-8);
    float2 delta = blur_axis.xy * offset * blur_texel.xy;
    total += (blur_source.SampleLevel(linear_sampler, uv + delta, 0)
      + blur_source.SampleLevel(linear_sampler, uv - delta, 0)) * weight;
    total_weight += weight * 2.0;
  }
  return total / total_weight;
}
