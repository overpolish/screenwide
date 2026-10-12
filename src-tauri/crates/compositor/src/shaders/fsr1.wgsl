// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-FileCopyrightText: 2021 Advanced Micro Devices, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later AND MIT

// AMD FidelityFX Super Resolution 1, as functions of the texture they read:
// RCAS, its sharpening, and EASU, its edge-aware enlargement. Ported from
// `ffx_fsr1.h`, whose notice follows.
//
// Copyright (c) 2021 Advanced Micro Devices, Inc. All rights reserved.
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

fn fsr_texel(image: texture_2d<f32>, at: vec2<i32>) -> vec3<f32> {
  let last = vec2<i32>(textureDimensions(image)) - 1;
  return textureLoad(image, clamp(at, vec2<i32>(0), last), 0).rgb;
}

// FSR's luma, twice the usual scale, which both passes weigh by.
fn fsr_luma(colour: vec3<f32>) -> f32 {
  return colour.b * 0.5 + (colour.r * 0.5 + colour.g);
}

// The pixel `at` sharpened at RCAS's full strength. Its lobe is capped, and
// limited so no pixel is pushed to clip, which keeps halos faint; and it
// holds back where the neighbourhood looks like noise rather than detail.
fn fsr_rcas(image: texture_2d<f32>, at: vec2<i32>) -> vec3<f32> {
  let b = fsr_texel(image, at + vec2<i32>(0, -1));
  let d = fsr_texel(image, at + vec2<i32>(-1, 0));
  let e = fsr_texel(image, at);
  let f = fsr_texel(image, at + vec2<i32>(1, 0));
  let h = fsr_texel(image, at + vec2<i32>(0, 1));
  let lowest = min(min(b, d), min(f, h));
  let highest = max(max(b, d), max(f, h));
  let hit_min = min(lowest, e) / max(4.0 * highest, vec3<f32>(1e-6));
  let hit_max = (1.0 - max(highest, e)) / min(4.0 * lowest - 4.0, vec3<f32>(-1e-6));
  let lobes = max(-hit_min, hit_max);
  var lobe = max(-(0.25 - 1.0 / 16.0), min(max(lobes.r, max(lobes.g, lobes.b)), 0.0));
  let lb = fsr_luma(b);
  let ld = fsr_luma(d);
  let le = fsr_luma(e);
  let lf = fsr_luma(f);
  let lh = fsr_luma(h);
  let spread = max(max(max(lb, ld), max(lf, lh)), le) - min(min(min(lb, ld), min(lf, lh)), le);
  let noise = saturate(abs(0.25 * (lb + ld + lf + lh) - le) / max(spread, 1e-6));
  lobe *= 1.0 - 0.5 * noise;
  return saturate((lobe * (b + d + f + h) + e) / (4.0 * lobe + 1.0));
}

struct FsrDirection {
  direction: vec2<f32>,
  length: f32,
}

// One corner of the bilinear footprint's say in the edge's direction and how
// sharp it is, from the cross of lumas around it: `c` the corner, `a` and
// `e` above and below it, `b` and `d` either side.
fn fsr_easu_corner(sum: FsrDirection, weight: f32, a: f32, b: f32, c: f32, d: f32, e: f32)
    -> FsrDirection {
  let across = d - b;
  let along_x = saturate(abs(across) / max(max(abs(d - c), abs(c - b)), 1e-8));
  let down = e - a;
  let along_y = saturate(abs(down) / max(max(abs(e - c), abs(c - a)), 1e-8));
  return FsrDirection(sum.direction + vec2<f32>(across, down) * weight,
                      sum.length + (along_x * along_x + along_y * along_y) * weight);
}

// `image` at `uv`, in shares of it, enlarged along its edges: twelve texels
// weighed by a Lanczos-like lobe stretched along the local edge and held
// within the four nearest, so it does not ring. For drawing larger only.
fn fsr_easu(image: texture_2d<f32>, uv: vec2<f32>) -> vec3<f32> {
  let point = uv * vec2<f32>(textureDimensions(image)) - 0.5;
  let base = floor(point);
  let pp = point - base;
  let at = vec2<i32>(base);
  let offsets = array<vec2<i32>, 12>(
    vec2(0, -1), vec2(1, -1), vec2(-1, 0), vec2(0, 0), vec2(1, 0), vec2(2, 0),
    vec2(-1, 1), vec2(0, 1), vec2(1, 1), vec2(2, 1), vec2(0, 2), vec2(1, 2));
  var colours: array<vec3<f32>, 12>;
  var lumas: array<f32, 12>;
  for (var tap = 0; tap < 12; tap++) {
    colours[tap] = fsr_texel(image, at + offsets[tap]);
    lumas[tap] = fsr_luma(colours[tap]);
  }
  // Taps by name, as the reference has them: b c / e f g h / i j k l / n o.
  var sum = FsrDirection(vec2<f32>(0.0), 0.0);
  sum = fsr_easu_corner(sum, (1.0 - pp.x) * (1.0 - pp.y), lumas[0], lumas[2], lumas[3], lumas[4], lumas[7]);
  sum = fsr_easu_corner(sum, pp.x * (1.0 - pp.y), lumas[1], lumas[3], lumas[4], lumas[5], lumas[8]);
  sum = fsr_easu_corner(sum, (1.0 - pp.x) * pp.y, lumas[3], lumas[6], lumas[7], lumas[8], lumas[10]);
  sum = fsr_easu_corner(sum, pp.x * pp.y, lumas[4], lumas[7], lumas[8], lumas[9], lumas[11]);
  let squared = dot(sum.direction, sum.direction);
  let flat_area = squared < 1.0 / 32768.0;
  let direction = select(sum.direction * inverseSqrt(max(squared, 1e-12)), vec2<f32>(1.0, 0.0), flat_area);
  var sharpness = sum.length * 0.5;
  sharpness *= sharpness;
  let stretch = dot(direction, direction) / max(abs(direction.x), abs(direction.y));
  let scale = vec2<f32>(1.0 + (stretch - 1.0) * sharpness, 1.0 - 0.5 * sharpness);
  let lobe = 0.5 + ((1.0 / 4.0 - 0.04) - 0.5) * sharpness;
  let clip = 1.0 / lobe;
  var total = vec3<f32>(0.0);
  var weights = 0.0;
  for (var tap = 0; tap < 12; tap++) {
    let offset = vec2<f32>(offsets[tap]) - pp;
    let v = vec2<f32>(dot(offset, direction), dot(offset, vec2<f32>(-direction.y, direction.x))) * scale;
    let distance = min(dot(v, v), clip);
    var window = 2.0 / 5.0 * distance - 1.0;
    var base_lobe = lobe * distance - 1.0;
    window *= window;
    base_lobe *= base_lobe;
    window = 25.0 / 16.0 * window - (25.0 / 16.0 - 1.0);
    let weight = window * base_lobe;
    total += colours[tap] * weight;
    weights += weight;
  }
  let lowest = min(min(colours[3], colours[4]), min(colours[7], colours[8]));
  let highest = max(max(colours[3], colours[4]), max(colours[7], colours[8]));
  return clamp(total / weights, lowest, highest);
}
