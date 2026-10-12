// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Enhance's colour correction, as `camera_effects/look.rs` works it on the
// CPU: the recording's white balance and exposure as gains in linear light,
// then the Natural look on display values, as a grading LUT works. The look
// moves brightness for everyone alike and leaves skin's hue and saturation
// mostly as they are; skin is told by hue and chroma alone, which every
// skin tone shares.

// The vectorscope's skin tone line, 123 degrees, in the Cb/Cr plane.
const correction_skin_hue: f32 = 2.1467549;
const correction_luma: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);

fn correction_linear(value: vec3<f32>) -> vec3<f32> {
  return select(pow((value + 0.055) / 1.055, vec3<f32>(2.4)), value / 12.92, value <= vec3<f32>(0.04045));
}

fn correction_display(value: vec3<f32>) -> vec3<f32> {
  let clamped = saturate(value);
  return select(
    1.055 * pow(clamped, vec3<f32>(1.0 / 2.4)) - 0.055,
    clamped * 12.92,
    clamped <= vec3<f32>(0.0031308),
  );
}

fn correction_cbcr(rgb: vec3<f32>) -> vec2<f32> {
  return vec2<f32>(dot(rgb, vec3<f32>(-0.1146, -0.3854, 0.5)), dot(rgb, vec3<f32>(0.5, -0.4542, -0.0458)));
}

// How much a colour looks like skin, by hue and chroma alone.
fn correction_skin(c: vec2<f32>) -> f32 {
  let chroma = length(c);
  var off = atan2(c.y, c.x) - correction_skin_hue;
  off -= 6.2831853 * round(off / 6.2831853);
  let width = off / 0.31415927;
  return exp(-width * width) * smoothstep(0.012, 0.035, chroma) * (1.0 - smoothstep(0.14, 0.2, chroma));
}

// `rgb` with the Natural look: a gentle S-curve around the middle of
// brightness, a touch more saturation, and more for duller colours. Skin
// takes the look's brightness and keeps most of its own hue and saturation.
fn correction_natural(rgb: vec3<f32>) -> vec3<f32> {
  let contrast = 0.15;
  let saturation = 1.03;
  let vibrance = 0.12;
  let protect = 0.85;
  let c = correction_cbcr(rgb);
  let y0 = max(dot(rgb, correction_luma), 1e-5);
  let yy = saturate(y0 + contrast * y0 * (1.0 - y0) * (2.0 * y0 - 1.0));
  // Chroma follows brightness partway, so a darker shade is not also a duller one.
  let base = c * sqrt(clamp(yy / y0, 0.25, 4.0));
  let scale = saturation * (1.0 + vibrance * (1.0 - smoothstep(0.0, 0.22, length(base))));
  let shifted = mix(base * scale, base * saturation, correction_skin(c) * protect);
  let r = yy + 1.5748 * shifted.y;
  let b = yy + 1.8556 * shifted.x;
  let g = (yy - 0.2126 * r - 0.0722 * b) / 0.7152;
  return saturate(vec3<f32>(r, g, b));
}

// A display colour corrected: balanced by `gains` in linear light, then the
// Natural look.
fn colour_correction(rgb: vec3<f32>, gains: vec3<f32>) -> vec3<f32> {
  return correction_natural(correction_display(correction_linear(rgb) * gains));
}
