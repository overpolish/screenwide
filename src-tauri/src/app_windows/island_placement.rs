// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the island hangs: from the top of its display, under the notch
//! where there is one, or by a box on the desktop, as Text Recognition's
//! island hangs by the area it read. By a box, the island sits under it, or
//! over it where there is no room below, and its window spans the display so
//! the island can stay centred on the box however near an edge it is; the
//! page keeps it clear of the display's sides (`IslandBoxAnchor`).

use std::sync::{Mutex, PoisonError};

use serde::Serialize;

use super::island::IslandNotch;

/// The island's row: a control and the row's inset round it (`h-10` in
/// `island-row.tsx`). An island by a box shows its row alone.
const ROW_HEIGHT: f64 = 40.0;
/// Between the island and the box it hangs by, and between it and the
/// display's edges, as between neighbours that stand on their own
/// (`--spacing-separate`).
const GAP: f64 = 12.0;
/// Room above the island for its shadow (`ISLAND.shadowRoom`): the window's
/// top edge would otherwise cut the shadow straight across.
const SHADOW_ROOM: f64 = 48.0;

/// A rectangle in a display's own points, from its top left.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LocalRect {
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum IslandPlacement {
  /// At the top of the island's own display.
  Top,
  /// By `rect` on the display `display`.
  ByBox { display: u32, rect: LocalRect },
}

/// Where the page puts the island in a window placed by a box, in CSS
/// pixels from the window's top left: its top edge, the centre it is drawn
/// round, and the span it must stay within.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IslandBoxAnchor {
  pub top: f64,
  pub center_x: f64,
  pub min_x: f64,
  pub max_x: f64,
}

/// Where the island is to hang, and where its window was last put.
static PLACEMENT: Mutex<(IslandPlacement, Option<IslandPlacement>)> =
  Mutex::new((IslandPlacement::Top, None));

fn placement() -> std::sync::MutexGuard<'static, (IslandPlacement, Option<IslandPlacement>)> {
  PLACEMENT.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Sets where the island is to hang. Returns whether that changed.
pub(super) fn set(next: IslandPlacement) -> bool {
  let mut placement = placement();
  let changed = placement.0 != next;
  placement.0 = next;
  changed
}

pub(crate) fn current() -> IslandPlacement {
  placement().0
}

/// The window is being put where the island is to hang.
pub(super) fn mark_placed() {
  let mut placement = placement();
  placement.1 = Some(placement.0);
}

/// Whether the window stands somewhere other than where the island is to
/// hang, so showing it has to move it first.
pub(super) fn is_misplaced() -> bool {
  let placement = placement();
  placement.1 != Some(placement.0)
}

/// Where the island came out: under the notch of the screen it is on, if it
/// has one, or by a box.
#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct IslandShown {
  pub notch: Option<IslandNotch>,
  pub anchor: Option<IslandBoxAnchor>,
}

impl IslandShown {
  pub(super) const fn by_box(anchor: IslandBoxAnchor) -> Self {
    Self {
      notch: None,
      anchor: Some(anchor),
    }
  }

  #[cfg(target_os = "macos")]
  pub(super) const fn under(notch: Option<IslandNotch>) -> Self {
    Self {
      notch,
      anchor: None,
    }
  }
}

/// The island's window at the top of a display `display_width` points wide,
/// centred on it.
#[cfg(target_os = "macos")]
pub(super) fn top_frame(display_width: f64) -> LocalRect {
  let (width, height) = super::island::WINDOW_SIZE;
  LocalRect {
    x: (display_width - width) / 2.0,
    y: 0.0,
    width,
    height,
  }
}

/// The island's window by a box on a display `display_width` by
/// `display_height` points, in the display's points, and where the island
/// sits in it.
pub(crate) fn box_frame(
  display_width: f64,
  display_height: f64,
  window_height: f64,
  rect: LocalRect,
) -> (LocalRect, IslandBoxAnchor) {
  let below = rect.y + rect.height + GAP;
  let above = rect.y - GAP - ROW_HEIGHT;
  let island_top = if below + ROW_HEIGHT + GAP <= display_height {
    below
  } else if above >= GAP {
    above
  } else {
    // The box fills the display's height: inside it, along its bottom.
    (rect.y + rect.height - GAP - ROW_HEIGHT)
      .clamp(GAP, (display_height - ROW_HEIGHT - GAP).max(GAP))
  };
  let window_top = (island_top - SHADOW_ROOM).max(0.0);
  (
    LocalRect {
      x: 0.0,
      y: window_top,
      width: display_width,
      height: window_height,
    },
    IslandBoxAnchor {
      top: island_top - window_top,
      center_x: rect.x + rect.width / 2.0,
      min_x: GAP,
      max_x: display_width - GAP,
    },
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  const fn rect(x: f64, y: f64, width: f64, height: f64) -> LocalRect {
    LocalRect {
      x,
      y,
      width,
      height,
    }
  }

  #[test]
  fn the_island_hangs_under_a_box_with_room_below() {
    let (window, anchor) = box_frame(1440.0, 900.0, 640.0, rect(100.0, 200.0, 400.0, 300.0));
    assert_eq!(window, rect(0.0, 464.0, 1440.0, 640.0));
    assert_eq!(anchor.top, 48.0);
    assert_eq!(window.y + anchor.top, 512.0);
    assert_eq!(anchor.center_x, 300.0);
    assert_eq!((anchor.min_x, anchor.max_x), (12.0, 1428.0));
  }

  #[test]
  fn the_island_sits_over_a_box_that_reaches_the_bottom() {
    let (window, anchor) = box_frame(1440.0, 900.0, 640.0, rect(100.0, 500.0, 400.0, 380.0));
    assert_eq!(window.y + anchor.top, 448.0);
  }

  #[test]
  fn the_island_sits_inside_a_box_as_tall_as_the_display() {
    let (window, anchor) = box_frame(1440.0, 900.0, 640.0, rect(0.0, 0.0, 1440.0, 900.0));
    assert_eq!(window.y + anchor.top, 848.0);
  }

  #[test]
  fn the_window_keeps_to_the_display_top_and_the_shadow_room_shrinks() {
    let (window, anchor) = box_frame(1440.0, 900.0, 640.0, rect(100.0, 0.0, 400.0, 10.0));
    assert_eq!(window.y, 0.0);
    assert_eq!(anchor.top, 22.0);
  }
}
