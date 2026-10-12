// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Enhance's colour correction as each pixel takes it, the CPU twin of
//! `colour_correction.wgsl`: the recording's white balance and exposure as
//! gains in linear light, then the Natural look on display values, as a
//! grading LUT works.
//!
//! The look moves brightness for everyone alike and leaves skin's hue and
//! saturation mostly as they are. Skin is told by hue and chroma alone,
//! which every skin tone shares; only how dark it is differs between them.

/// Rec. 709 luma weights.
pub(super) const LUMA: [f32; 3] = [0.2126, 0.7152, 0.0722];
/// The vectorscope's skin tone line, in the Cb/Cr plane.
pub(super) const SKIN_HUE: f32 = 123.0 * std::f32::consts::PI / 180.0;

/// The Natural look: a gentle S-curve, a touch more saturation, and more
/// for duller colours; skin keeps this share of its own hue and saturation.
const CONTRAST: f32 = 0.15;
const SATURATION: f32 = 1.03;
const VIBRANCE: f32 = 0.12;
const PROTECT: f32 = 0.85;

pub(super) fn to_linear(value: f32) -> f32 {
  if value <= 0.040_45 {
    value / 12.92
  } else {
    ((value + 0.055) / 1.055).powf(2.4)
  }
}

pub(super) fn to_display(value: f32) -> f32 {
  let value = value.clamp(0.0, 1.0);
  if value <= 0.003_130_8 {
    value * 12.92
  } else {
    1.055 * value.powf(1.0 / 2.4) - 0.055
  }
}

/// A display colour's Cb and Cr.
pub(super) fn cbcr([r, g, b]: [f32; 3]) -> [f32; 2] {
  [
    -0.1146 * r - 0.3854 * g + 0.5 * b,
    0.5 * r - 0.4542 * g - 0.0458 * b,
  ]
}

fn smoothstep(low: f32, high: f32, x: f32) -> f32 {
  let t = ((x - low) / (high - low)).clamp(0.0, 1.0);
  t * t * (3.0 - 2.0 * t)
}

/// How much a colour looks like skin, by hue and chroma alone.
fn skin_weight([cb, cr]: [f32; 2]) -> f32 {
  let chroma = cb.hypot(cr);
  let off = cr.atan2(cb) - SKIN_HUE;
  let off = off - std::f32::consts::TAU * (off / std::f32::consts::TAU).round();
  (-(off / 18f32.to_radians()).powi(2)).exp()
    * smoothstep(0.012, 0.035, chroma)
    * (1.0 - smoothstep(0.14, 0.2, chroma))
}

/// `rgb` with the Natural look: an S-curve around the middle of
/// brightness, then saturation and vibrance. Skin takes the look's
/// brightness and keeps most of its own hue and saturation.
fn natural(rgb: [f32; 3]) -> [f32; 3] {
  let y = rgb
    .iter()
    .zip(LUMA)
    .map(|(value, weight)| value * weight)
    .sum::<f32>();
  let c = cbcr(rgb);
  let y0 = y.max(1e-5);
  let yy = (y0 + CONTRAST * y0 * (1.0 - y0) * (2.0 * y0 - 1.0)).clamp(0.0, 1.0);
  // Chroma follows brightness partway, so a darker shade is not also a duller one.
  let follow = (yy / y0).clamp(0.25, 4.0).sqrt();
  let base = c.map(|value| value * follow);
  let chroma = base[0].hypot(base[1]);
  let scale = SATURATION * (1.0 + VIBRANCE * (1.0 - smoothstep(0.0, 0.22, chroma)));
  let skin = skin_weight(c) * PROTECT;
  let [cb, cr] = base.map(|value| value * scale * (1.0 - skin) + value * SATURATION * skin);
  let r = yy + 1.5748 * cr;
  let b = yy + 1.8556 * cb;
  let g = (yy - LUMA[0] * r - LUMA[2] * b) / LUMA[1];
  [r, g, b].map(|value| value.clamp(0.0, 1.0))
}

/// A display colour, 0 to 1, corrected: balanced by `gains` in linear
/// light, then the Natural look.
pub(super) fn corrected(rgb: [f32; 3], gains: [f32; 3]) -> [f32; 3] {
  let mut balanced = [0.0; 3];
  for (index, value) in balanced.iter_mut().enumerate() {
    *value = to_display(to_linear(rgb[index]) * gains[index]);
  }
  natural(balanced)
}
