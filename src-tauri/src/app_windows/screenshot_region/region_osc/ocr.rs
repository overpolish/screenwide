// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The OCR overlay: the area read, the sheen that passes over it while it is
//! read, and the highlights on what was found. They join the scene's base
//! run in world space. What can be done with the result is the island's
//! (`text_recognition::island`), not drawn here.

mod sweep;
mod vertices;

use std::time::Instant;

use crate::text_recognition::visual::OcrRectPacket;
use screenwide_osc::geometry::{Point, Rect, Size};

/// `VisualPhase` values the compositor distinguishes.
pub(crate) const PHASE_LOADING: u32 = 1;
pub(crate) const PHASE_READY: u32 = 2;

/// One OCR highlight in surface-local points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LocalRect {
  pub rect: Rect,
  pub kind: u8,
}

/// `1 Line→17, 2 Qr→18, 3 QrError→19, 4 Selection→20`.
pub(crate) fn rect_kind(kind: u8) -> u32 {
  match kind {
    4 => 20,
    3 => 19,
    2 => 18,
    _ => 17,
  }
}

fn intersects(rect: Rect, bounds: Size) -> bool {
  rect.size.width > 0.0
    && rect.size.height > 0.0
    && rect.origin.x < bounds.width
    && rect.origin.y < bounds.height
    && rect.right() > 0.0
    && rect.bottom() > 0.0
}

/// Translates each desktop-space rect into this surface's own coordinates
/// and keeps only the ones that land on it.
pub(crate) fn local_rects(
  packets: &[OcrRectPacket],
  offset: Point,
  bounds: Size,
) -> Vec<LocalRect> {
  packets
    .iter()
    .filter_map(|packet| {
      let rect = Rect::from_xywh(
        packet.x - offset.x,
        packet.y - offset.y,
        packet.width,
        packet.height,
      );
      intersects(rect, bounds).then_some(LocalRect {
        rect,
        kind: packet.kind,
      })
    })
    .collect()
}

/// The sheen over the area: when reading began, which times it, and once
/// reading is done, when the crossing under way ends.
#[derive(Clone, Copy)]
struct Sheen {
  since: Instant,
  until: Option<Instant>,
}

impl Sheen {
  fn is_live(self, now: Instant) -> bool {
    self.until.is_none_or(|until| now < until)
  }
}

#[derive(Default)]
pub(crate) struct Chrome {
  pub phase: u32,
  rects: Vec<LocalRect>,
  sheen: Option<Sheen>,
}

impl Chrome {
  /// Stores this surface's share of the highlights for `phase`. Reading
  /// often takes less than one crossing, so when it is done the crossing
  /// under way runs on over what was found rather than being cut off.
  pub(crate) fn apply(
    &mut self,
    phase: u32,
    packets: &[OcrRectPacket],
    offset: Point,
    bounds: Size,
  ) {
    let now = Instant::now();
    self.sheen = match (phase, self.sheen) {
      (PHASE_LOADING, Some(sheen)) if sheen.until.is_none() => Some(sheen),
      (PHASE_LOADING, _) => Some(Sheen {
        since: now,
        until: None,
      }),
      (PHASE_READY, Some(sheen)) if self.phase == PHASE_LOADING => {
        let elapsed = now.saturating_duration_since(sheen.since).as_secs_f64();
        sweep::crossing_end(elapsed).map(|end| Sheen {
          until: Some(sheen.since + std::time::Duration::from_secs_f64(end)),
          ..sheen
        })
      }
      (PHASE_READY, sheen) => sheen,
      _ => None,
    };
    self.phase = phase;
    self.rects = local_rects(packets, offset, bounds);
  }

  /// The sheen moves while the area is read and while its last crossing
  /// runs on.
  pub(crate) fn is_animating(&self) -> bool {
    self
      .sheen
      .is_some_and(|sheen| sheen.is_live(Instant::now()))
  }
}

#[cfg(test)]
mod tests;
