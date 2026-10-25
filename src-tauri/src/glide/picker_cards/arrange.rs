// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where each card goes: the arrangement centred on the anchor, kept inside
//! the work area of the monitor the anchor is on, in that monitor's physical
//! pixels so a card on a differently scaled monitor is sized for it.

use tauri::{AppHandle, Monitor, PhysicalPosition, PhysicalSize, Position, Size};

use crate::glide::core::monitors::{CARD, CARD_GAP};
use crate::glide::SHADOW_ROOM;

/// Offsets for `count` cards in a row, a gap apart: the Spaces picker.
pub(crate) fn row(count: usize) -> Vec<(f64, f64)> {
  (0..count)
    .map(|index| (index as f64 * (CARD.0 + CARD_GAP), 0.0))
    .collect()
}

/// A card's window, in logical points: the card with its shadow room round
/// it. Neighbouring windows overlap by the room, harmlessly, since they ignore
/// the pointer.
pub(super) const WINDOW: (f64, f64) = (CARD.0 + SHADOW_ROOM * 2.0, CARD.1 + SHADOW_ROOM * 2.0);

/// Where a gesture is anchored, in the units its platform reports.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Anchor {
  /// Logical points, as macOS reports the pointer.
  #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
  Logical(f64, f64),
  /// Physical pixels, as Windows reports the pointer.
  #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
  Physical(f64, f64),
}

pub(super) struct Placement {
  pub position: Position,
  pub size: Size,
}

/// The monitor `anchor` lies on and the anchor in physical pixels.
fn monitor_at(app: &AppHandle, anchor: Anchor) -> Option<(Monitor, (f64, f64))> {
  let monitors = app.available_monitors().ok()?;
  monitors.into_iter().find_map(|monitor| {
    let origin = monitor.position();
    let size = monitor.size();
    let scale = monitor.scale_factor();
    let physical = match anchor {
      Anchor::Physical(x, y) => (x, y),
      Anchor::Logical(x, y) => {
        let logical = origin.to_logical::<f64>(scale);
        (
          f64::from(origin.x) + (x - logical.x) * scale,
          f64::from(origin.y) + (y - logical.y) * scale,
        )
      }
    };
    let inside = physical.0 >= f64::from(origin.x)
      && physical.0 < f64::from(origin.x) + f64::from(size.width)
      && physical.1 >= f64::from(origin.y)
      && physical.1 < f64::from(origin.y) + f64::from(size.height);
    inside.then_some((monitor, physical))
  })
}

/// One placement per offset, the offsets in logical points from the
/// arrangement's corner to each card. The cards are centred on the anchor and
/// kept in the work area; their windows reach past them by the shadow room.
pub(super) fn place(app: &AppHandle, anchor: Anchor, offsets: &[(f64, f64)]) -> Vec<Placement> {
  let found = monitor_at(app, anchor);
  let scale = found
    .as_ref()
    .map_or(1.0, |(monitor, _)| monitor.scale_factor());
  let (x, y) = found.as_ref().map_or(
    match anchor {
      Anchor::Logical(x, y) | Anchor::Physical(x, y) => (x, y),
    },
    |(_, physical)| *physical,
  );
  let width = (offsets.iter().map(|offset| offset.0).fold(0.0, f64::max) + CARD.0) * scale;
  let height = (offsets.iter().map(|offset| offset.1).fold(0.0, f64::max) + CARD.1) * scale;
  let (mut left, mut top) = (x - width / 2.0, y - height / 2.0);
  if let Some((monitor, _)) = found.as_ref() {
    let work = monitor.work_area();
    let (work_x, work_y) = (f64::from(work.position.x), f64::from(work.position.y));
    let (work_width, work_height) = (f64::from(work.size.width), f64::from(work.size.height));
    left = left.clamp(work_x, (work_x + work_width - width).max(work_x));
    top = top.clamp(work_y, (work_y + work_height - height).max(work_y));
  }
  let size = PhysicalSize::new(
    (WINDOW.0 * scale).round() as u32,
    (WINDOW.1 * scale).round() as u32,
  );
  offsets
    .iter()
    .map(|(offset_x, offset_y)| Placement {
      position: Position::Physical(PhysicalPosition::new(
        (left + (offset_x - SHADOW_ROOM) * scale).round() as i32,
        (top + (offset_y - SHADOW_ROOM) * scale).round() as i32,
      )),
      size: Size::Physical(size),
    })
    .collect()
}
