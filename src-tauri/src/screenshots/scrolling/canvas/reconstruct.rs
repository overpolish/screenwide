// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::screenshots::CapturedImage;

/// Per-channel distance at which two tiles are read as showing the same
/// document pixel: wide enough to absorb capture noise and the compression a
/// screen capture applies, narrow enough that a scrollbar thumb never matches
/// the page underneath it.
const AGREEMENT_TOLERANCE: i16 = 12;

/// One tile's placement in canvas coordinates, carrying the crop bounds the
/// placement loop honours so that a cropped-away region contributes nothing.
pub(super) struct Coverage<'a> {
  pub(super) column_end: u32,
  pub(super) column_start: u32,
  pub(super) image: &'a CapturedImage,
  pub(super) origin_x: u32,
  pub(super) origin_y: u32,
  pub(super) row_end: u32,
  pub(super) row_start: u32,
}

impl Coverage<'_> {
  fn canvas_row_end(&self) -> u32 {
    self.origin_y.saturating_add(self.row_end)
  }

  fn canvas_row_start(&self) -> u32 {
    self.origin_y.saturating_add(self.row_start)
  }

  fn sample(&self, x: u32, y: u32) -> Option<[u8; 4]> {
    let column = x.checked_sub(self.origin_x)?;
    let row = y.checked_sub(self.origin_y)?;
    if column < self.column_start
      || column >= self.column_end
      || row < self.row_start
      || row >= self.row_end
    {
      return None;
    }
    let offset = ((row * self.image.width + column) * 4) as usize;
    self.image.rgba.get(offset..offset + 4)?.try_into().ok()
  }
}

fn agrees(left: [u8; 4], right: [u8; 4]) -> bool {
  left
    .iter()
    .zip(right)
    .all(|(left, right)| (i16::from(*left) - i16::from(right)).abs() <= AGREEMENT_TOLERANCE)
}

/// The value at least two covering tiles agree on, verbatim from the first of
/// them. Averaging would nudge genuine content, so the agreed candidate is
/// copied unchanged; the odd one out - the tile whose thumb covers this
/// document row - is simply never the one that finds a partner.
fn agreed_value(candidates: &[[u8; 4]]) -> Option<[u8; 4]> {
  candidates.iter().copied().find(|candidate| {
    candidates
      .iter()
      .filter(|other| agrees(*candidate, **other))
      .count()
      >= 2
  })
}

/// Rebuilds the right-edge strip from tile overlap.
///
/// An overlay scrollbar sits at a fixed viewport x but moves along the scroll
/// axis between frames, while the document does not, so consecutive tiles
/// disagree exactly where the thumb is and agree everywhere else. Any pixel two
/// tiles agree on is the document and is written back; anything else is left as
/// the placement loop wrote it.
///
/// Rows only one tile covers - the head of the first tile and the tail of the
/// last - have nothing to compare against, so a thumb remnant can survive
/// there. That is the deliberate residual: it beats erasing real content, such
/// as a source-control panel's status letters, from every row.
pub(super) fn reconstruct_right_edge(
  rgba: &mut [u8],
  width: u32,
  height: u32,
  strip_start: u32,
  coverage: &[Coverage<'_>],
) {
  // Bounds-testing every tile per pixel would cost tens of millions of tests on
  // a tall capture, so tiles enter and leave a small active set as the sweep
  // crosses their row range.
  let mut order: Vec<usize> = (0..coverage.len()).collect();
  order.sort_by_key(|index| coverage[*index].canvas_row_start());
  let mut pending = 0;
  let mut active: Vec<usize> = Vec::new();
  let mut candidates: Vec<[u8; 4]> = Vec::new();
  for y in 0..height {
    while pending < order.len() && coverage[order[pending]].canvas_row_start() <= y {
      active.push(order[pending]);
      pending += 1;
    }
    active.retain(|index| coverage[*index].canvas_row_end() > y);
    if active.len() < 2 {
      continue;
    }
    for x in strip_start..width {
      candidates.clear();
      candidates.extend(
        active
          .iter()
          .filter_map(|index| coverage[*index].sample(x, y)),
      );
      let Some(agreed) = agreed_value(&candidates) else {
        continue;
      };
      let destination = ((y * width + x) * 4) as usize;
      rgba[destination..destination + 4].copy_from_slice(&agreed);
    }
  }
}
