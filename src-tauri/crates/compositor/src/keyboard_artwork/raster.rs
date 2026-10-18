// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The shortcut strip's artwork, the same on every platform: the React
//! keycap's `h-5 min-w-5 gap-tight rounded-sm bg-fill px-control` caps,
//! labelled in Inter Regular at 13 points with tabular figures, set by the
//! platform's text engine, or with the modifier's icon. Premultiplied grey,
//! so it reads the same as BGRA or RGBA.

mod icons;
mod labels;
#[cfg(test)]
mod tests;

use super::{KeyboardRaster, DESIGN_HEIGHT};
use crate::platform::type_device::TypeDevice;
use labels::Cap;
use screenwide_text::engine::face::TypeFace;

const DESIGN_MINIMUM_WIDTH: f64 = 20.0;
const DESIGN_INSET: f64 = 4.0;
const DESIGN_GAP: f64 = 4.0;
const DESIGN_RADIUS: f64 = 4.0;
const FONT_SIZE: f64 = 13.0;
const FILL_ALPHA: f64 = 0.10;
const TEXT_ALPHA: f64 = 0.85;

/// The strip for the keys `codes`, in the light or dark appearance, at
/// `backing_scale` pixels per design point.
pub(crate) fn rasterize_keyboard(
  codes: &[u16],
  light: bool,
  backing_scale: f64,
) -> Result<KeyboardRaster, String> {
  let caps = codes
    .iter()
    .map(|code| labels::cap(*code))
    .collect::<Vec<_>>();
  rasterize_caps(&caps, light, backing_scale)
}

fn rasterize_caps(caps: &[Cap], light: bool, backing_scale: f64) -> Result<KeyboardRaster, String> {
  if caps.is_empty() {
    return Err("The keyboard shortcut has no keys to draw".to_owned());
  }
  let device =
    TypeDevice::tabular_in_face(TypeFace::new(None, 400, false), FONT_SIZE * backing_scale)?;
  let content_widths = caps
    .iter()
    .map(|cap| match cap {
      Cap::Text(text) => device.advance(text) / backing_scale,
      Cap::Icon(_) => icons::ICON_SIZE,
    })
    .collect::<Vec<_>>();
  let key_widths = content_widths
    .iter()
    .map(|width| (width.ceil() + DESIGN_INSET * 2.0).max(DESIGN_MINIMUM_WIDTH))
    .collect::<Vec<_>>();
  let design_width =
    key_widths.iter().sum::<f64>() + DESIGN_GAP * (key_widths.len().saturating_sub(1)) as f64;
  let width = ((design_width * backing_scale).ceil() as u32).max(1);
  let height = ((DESIGN_HEIGHT * backing_scale).ceil() as u32).max(1);

  let mut keys = Vec::with_capacity(caps.len());
  let mut lines = Vec::with_capacity(caps.len());
  let (ascent, descent) = device.vertical_metrics();
  let text_top = (f64::from(height) - (ascent + descent)) * 0.5;
  let mut key_x = 0.0_f64;
  for ((cap, key_width), content_width) in caps.iter().zip(&key_widths).zip(&content_widths) {
    if let Cap::Text(text) = cap {
      let text_x = key_x + (key_width - content_width) * 0.5;
      lines.push(((text_x * backing_scale, text_top), text.as_str()));
    }
    keys.push((
      (key_x * backing_scale).round() as u32,
      (key_width * backing_scale).round() as u32,
    ));
    key_x += key_width + DESIGN_GAP;
  }
  let mut coverage = device.draw((width, height), &lines)?;
  for (cap, key) in caps.iter().zip(&keys) {
    if let Cap::Icon(icon) = cap {
      icons::draw(&mut coverage, (width, height), *key, *icon, backing_scale);
    }
  }
  Ok(KeyboardRaster {
    pixels: compose(&coverage, (width, height), &keys, light, backing_scale),
    size: (width, height),
    keys,
  })
}

/// The caps filled and the labels' `coverage` laid over them, premultiplied.
fn compose(
  coverage: &[u8],
  (width, height): (u32, u32),
  keys: &[(u32, u32)],
  light: bool,
  backing_scale: f64,
) -> Vec<u8> {
  // Resolve bg-fill over the Storybook window backing (#FFFFFF / #252525):
  // one opaque shape keeps its antialiased edge and the gaps clean.
  let (backing, foreground) = if light { (255.0, 0.0) } else { (37.0, 255.0) };
  let background = backing * (1.0 - FILL_ALPHA) + foreground * FILL_ALPHA;
  let radius = DESIGN_RADIUS * backing_scale;
  let mut pixels = vec![0_u8; width as usize * height as usize * 4];
  for (cap_x, cap_pixels) in keys {
    let left = f64::from(*cap_x);
    let cap_width = f64::from(*cap_pixels);
    let first = cap_x.saturating_sub(2);
    let last = (cap_x + cap_pixels + 2).min(width);
    for row in 0..height {
      let y = f64::from(row) + 0.5;
      for column in first..last {
        let cap = rounded_coverage(
          f64::from(column) + 0.5,
          y,
          left,
          cap_width,
          f64::from(height),
          radius,
        );
        if cap <= 0.0 {
          continue;
        }
        let offset = ((row as usize) * (width as usize) + column as usize) * 4;
        let value = (background * cap).round().clamp(0.0, 255.0) as u8;
        pixels[offset..offset + 3].fill(value);
        pixels[offset + 3] = (cap * 255.0).round().clamp(0.0, 255.0) as u8;
      }
    }
  }
  for (pixel, text) in pixels.chunks_exact_mut(4).zip(coverage) {
    if *text == 0 {
      continue;
    }
    let text = f64::from(*text) / 255.0 * TEXT_ALPHA;
    let keep = 1.0 - text;
    let alpha = text + (f64::from(pixel[3]) / 255.0) * keep;
    for channel in pixel[..3].iter_mut() {
      let value = foreground * text + f64::from(*channel) * keep;
      *channel = value.round().clamp(0.0, 255.0) as u8;
    }
    pixel[3] = (alpha * 255.0).round().clamp(0.0, 255.0) as u8;
  }
  pixels
}

/// How much of the pixel at `(x, y)` the rounded cap starting at `left`
/// covers.
fn rounded_coverage(x: f64, y: f64, left: f64, width: f64, height: f64, radius: f64) -> f64 {
  let half_width = width * 0.5;
  let half_height = height * 0.5;
  let radius = radius.min(half_width).min(half_height).max(0.0);
  let local_x = (x - (left + half_width)).abs() - (half_width - radius);
  let local_y = (y - half_height).abs() - (half_height - radius);
  let outside = local_x.max(0.0).hypot(local_y.max(0.0));
  let distance = outside + local_x.max(local_y).min(0.0) - radius;
  (0.5 - distance).clamp(0.0, 1.0)
}
