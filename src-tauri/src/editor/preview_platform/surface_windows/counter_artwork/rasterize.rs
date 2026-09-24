// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! DirectWrite rasterisation of the counters' numbers, the twin of the Core
//! Text pass in `gpu_compositor_macos_annotation_text.m`, and the pixels
//! every piece of annotation type is handed to the atlas as.

use super::*;
use crate::editor::preview_platform::surface::type_device::TypeDevice;

/// One number set for its cell: the size it is drawn at, its advance and
/// line height there, and the cell it is centred in. A number too wide for
/// its disc is narrowed rather than allowed to touch the edge.
struct Row {
  size: f64,
  measured: (f64, f64),
  cell: (u32, u32),
}

fn row(value: &str, radius: f32) -> Result<Row, String> {
  let diameter = f64::from(radius) * 2.0 * SUPERSAMPLE;
  let mut size = diameter * CAP_SHARE / CAP_HEIGHT;
  let device = TypeDevice::numbers(size)?;
  let (ascent, descent) = device.vertical_metrics();
  let mut measured = (device.advance(value), ascent + descent);
  if measured.0 <= 0.0 {
    return Err("Windows could not measure annotation type".to_owned());
  }
  let limit = diameter * WIDTH_SHARE;
  if measured.0 > limit {
    // The type scales evenly with its size, so the narrowed number needs no
    // second measurement.
    let scale = limit / measured.0;
    size *= scale;
    measured = (measured.0 * scale, measured.1 * scale);
  }
  // One transparent pixel of margin keeps the number's edge off the cell's.
  let cell = (
    (measured.0.ceil() as u32).saturating_add(2),
    (measured.1.ceil() as u32).saturating_add(2),
  );
  Ok(Row {
    size,
    measured,
    cell,
  })
}

/// The cell `value` needs at `radius`, in atlas pixels.
pub(super) fn measure(value: &str, radius: f32) -> Result<(u32, u32), String> {
  Ok(row(value, radius)?.cell)
}

/// `value` drawn at `radius` into a cell of `cell` pixels, as BGRA rows, top
/// row first.
pub(super) fn draw(value: &str, radius: f32, cell: (u32, u32)) -> Result<Vec<u8>, String> {
  let row = row(value, radius)?;
  let device = TypeDevice::numbers(row.size)?;
  let at = (
    (f64::from(cell.0) - row.measured.0) / 2.0,
    (f64::from(cell.1) - row.measured.1) / 2.0,
  );
  Ok(tinted(&device.draw(cell, &[(at, value)])?))
}

/// Coverage as the atlas's BGRA pixels. The type is tinted by the shader, so
/// its coverage is carried in the alpha channel over white.
pub(super) fn tinted(coverage: &[u8]) -> Vec<u8> {
  coverage
    .iter()
    .flat_map(|&covered| [255, 255, 255, covered])
    .collect()
}
