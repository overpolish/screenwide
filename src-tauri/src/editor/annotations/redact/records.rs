// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A composition's redactions as the Windows compositor's passes read them:
//! each box in whole source pixels, snapped outward and clipped to the
//! source, with how it is painted and the grid it draws from. The twin of
//! `screenwide_redactions` in `gpu_compositor_macos_redact.m`, which packs
//! the same records for Metal.

use crate::editor::annotations::flags::{BLUR, MOSAIC, PIXELATE};
use crate::editor::annotations::native::NativeAnnotation;
use crate::editor::annotations::AnnotationKind;

/// How a redaction's covered pixels are painted, as the passes branch on it.
pub(crate) const REDACT_FLAT: u32 = 0;
pub(crate) const REDACT_PIXELATE: u32 = 1;
pub(crate) const REDACT_BLUR: u32 = 2;
pub(crate) const REDACT_MOSAIC: u32 = 3;

/// The most cells the cells pass averages for one box. Rust sizes the cells
/// well under this; a ramp's first small cells are grown to stay within it.
const MAX_CELLS: f32 = (1u32 << 20) as f32;

/// One redaction, the twin of the `Redaction` constant buffer in
/// `redact.hlsl`: 16-byte rows, since a constant buffer packs by them.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct RedactRecord {
  /// `[x0, y0, x1, y1)` in whole source pixels.
  pub(crate) bounds: [u32; 4],
  pub(crate) source_width: u32,
  pub(crate) mode: u32,
  pub(crate) seed: u32,
  /// A pixelated box's block, or a blurred or classically pixelated box's
  /// cell, in source pixels.
  pub(crate) size: f32,
  /// The fill, whose alpha is the share an arriving fill has reached.
  pub(crate) color: [f32; 4],
  /// A pixelated box's zones across and blocks per zone, or a classically
  /// pixelated or blurred box's cells across and down.
  pub(crate) grid: [u32; 2],
  pub(crate) entry_count: u32,
  /// The corner radius in source pixels.
  pub(crate) radius: f32,
  /// Where this box's zones start in the list's zones.
  pub(crate) zone_first: u32,
  pub(crate) padding: [u32; 3],
}

const _: () = assert!(std::mem::size_of::<RedactRecord>() == 80);

/// A composition's redactions in draw order, and the zones they index.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct RedactRecords {
  pub(crate) records: Vec<RedactRecord>,
  pub(crate) zones: Vec<[f32; 2]>,
}

/// A corner coordinate as a whole pixel inside `[0, limit]`: down at the
/// top-left and up at the bottom-right, so no covered pixel is left partly
/// showing along the edge.
fn edge(value: f32, far: bool, limit: u32) -> u32 {
  let edge = if far { value.ceil() } else { value.floor() };
  if !(edge > 0.0) {
    return 0;
  }
  if edge >= limit as f32 {
    limit
  } else {
    edge as u32
  }
}

/// A whole count a record carries as a float, or zero for one it cannot.
fn count(value: f32) -> u32 {
  if (1.0..16_777_216.0).contains(&value) {
    value as u32
  } else {
    0
  }
}

/// The redactions among `items`, over a `width` by `height` source whose
/// side buffer is `points`.
pub(crate) fn redact_records(
  items: &[NativeAnnotation],
  points: &[[f32; 2]],
  width: u32,
  height: u32,
) -> RedactRecords {
  let mut list = RedactRecords::default();
  for item in items {
    if AnnotationKind::from_raw(item.kind) != Some(AnnotationKind::Redact) {
      continue;
    }
    let mode = if item.flags & PIXELATE != 0 {
      REDACT_PIXELATE
    } else if item.flags & BLUR != 0 {
      REDACT_BLUR
    } else if item.flags & MOSAIC != 0 {
      REDACT_MOSAIC
    } else {
      REDACT_FLAT
    };
    // `p0` is the top-left corner and `p2` the bottom-right.
    let bounds = [
      edge(item.p0[0], false, width),
      edge(item.p0[1], false, height),
      edge(item.p2[0], true, width),
      edge(item.p2[1], true, height),
    ];
    if bounds[2] <= bounds[0] || bounds[3] <= bounds[1] {
      continue;
    }
    let (wide, tall) = (
      (bounds[2] - bounds[0]) as f32,
      (bounds[3] - bounds[1]) as f32,
    );
    // The record's width is the radius as a share of the painted box's
    // shorter side, so the halo drawn from the unsnapped box rounds alike.
    let share = if item.width.is_finite() {
      item.width.clamp(0.0, 50.0)
    } else {
      0.0
    };
    let mut record = RedactRecord {
      bounds,
      source_width: width,
      mode,
      // The seed rides in two float halves, each exact.
      seed: ((item.params[1] as u32) << 16) | (item.params[2] as u32 & 0xffff),
      size: item.params[0],
      color: item.color,
      radius: wide.min(tall) * share / 100.0,
      zone_first: list.zones.len() as u32,
      ..RedactRecord::default()
    };
    // An animated redaction arrives by covering more: a classic pixelation's
    // blocks and a blur's cells grow from a pixel to their size, and every
    // other fill fades in, its share riding in the colour's alpha.
    let arrived = if item.reveal.opacity.is_finite() {
      item.reveal.opacity.clamp(0.0, 1.0)
    } else {
      1.0
    };
    if matches!(mode, REDACT_BLUR | REDACT_MOSAIC) {
      record.color[3] = 1.0;
      let size = if record.size.is_finite() {
        record.size.max(1.0)
      } else {
        1.0
      };
      let size = (1.0 + (size - 1.0) * arrived).max((wide * tall / MAX_CELLS).sqrt());
      record.size = size;
      record.grid = [
        (wide / size).ceil().max(1.0) as u32,
        (tall / size).ceil().max(1.0) as u32,
      ];
    } else {
      record.color[3] = arrived;
      let first = item.data_offset as usize;
      let zones = first
        .checked_add(item.data_count as usize)
        .and_then(|last| points.get(first..last));
      if let (REDACT_PIXELATE, Some(zones)) = (mode, zones) {
        record.grid = [count(item.p3[0]), count(item.p3[1])];
        if record.grid[0] > 0 && record.grid[1] > 0 && !zones.is_empty() {
          record.entry_count = zones.len() as u32;
          list.zones.extend_from_slice(zones);
        }
      }
    }
    list.records.push(record);
  }
  list
}

#[cfg(test)]
#[path = "records_tests.rs"]
mod tests;
