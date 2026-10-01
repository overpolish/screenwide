// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The display the toolbar belongs to, and where on it the toolbar sits.

use std::sync::{Mutex, MutexGuard};

use tauri::{AppHandle, LogicalPosition, LogicalSize};

use super::settings;
use crate::capture_overlays;

/// How far below the top of the display the toolbar opens.
const TOP_GAP: f64 = 16.0;

/// What the window is built at before the plate reports what it needs. Never
/// seen: the window stays hidden until the first fit has landed.
pub(super) const INITIAL_SIZE: LogicalSize<f64> = LogicalSize {
  width: 420.0,
  height: 48.0,
};

/// The display the toolbar opens on, which is the overlay's anchor host.
/// `origin` and `size` are its usable area: a toolbar opened top-centre has
/// to clear the menu bar and the notch, and a dropped place is measured from
/// that corner. `screen_origin` and `screen_size` are the whole display,
/// which is what a dropped toolbar is kept inside, as the recording bar is.
/// Kept from the build so a later fit can place the window again without
/// reading the monitor layout a second time.
pub(super) struct Anchor {
  pub(super) display_id: u32,
  pub(super) origin: LogicalPosition<f64>,
  pub(super) size: LogicalSize<f64>,
  pub(super) screen_origin: LogicalPosition<f64>,
  pub(super) screen_size: LogicalSize<f64>,
}

static ANCHOR: Mutex<Option<Anchor>> = Mutex::new(None);

pub(super) fn current() -> MutexGuard<'static, Option<Anchor>> {
  ANCHOR
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Where a toolbar of `size` belongs on the anchor display: where it was last
/// dropped there, or top-centre.
pub(super) fn placement(anchor: &Anchor, size: LogicalSize<f64>) -> LogicalPosition<f64> {
  placed(anchor, settings::current().toolbar_position, size)
}

/// Where a toolbar of `size` sits given the place it was last dropped, if
/// any. A place on another display is not this one's, so the toolbar opens
/// top-centre. A place the toolbar no longer fits in is kept as near as it
/// fits on the screen, the bounds a drop is held to: a tool with more
/// controls widens the plate to the right, and one near the right edge
/// slides left rather than jumping to the middle. The remembered place itself
/// is left alone, so a narrower tool puts the toolbar back where it was
/// dropped.
fn placed(
  anchor: &Anchor,
  remembered: Option<settings::ToolbarPosition>,
  size: LogicalSize<f64>,
) -> LogicalPosition<f64> {
  let Some(position) = remembered.filter(|position| position.display_id == anchor.display_id)
  else {
    return LogicalPosition::new(
      anchor.origin.x + ((anchor.size.width - size.width) / 2.0).max(0.0),
      anchor.origin.y + TOP_GAP,
    );
  };
  let within =
    |at: f64, start: f64, room: f64, extent: f64| at.clamp(start, start + (room - extent).max(0.0));
  LogicalPosition::new(
    within(
      anchor.origin.x + position.x,
      anchor.screen_origin.x,
      anchor.screen_size.width,
      size.width,
    ),
    within(
      anchor.origin.y + position.y,
      anchor.screen_origin.y,
      anchor.screen_size.height,
      size.height,
    ),
  )
}

/// The display the toolbar's top-left corner sits on, with the origin of its
/// usable area: that is the corner [`placement`] measures from, so a drop and
/// the place it is restored to are the same offset.
pub(super) fn display_under(
  app: &AppHandle,
  position: LogicalPosition<f64>,
) -> Result<Option<(u32, LogicalPosition<f64>)>, String> {
  for (display_id, scale, monitor) in capture_overlays::monitor_layout(app)? {
    let origin = monitor.position().to_logical::<f64>(scale);
    let size = monitor.size().to_logical::<f64>(scale);
    if position.x >= origin.x
      && position.y >= origin.y
      && position.x < origin.x + size.width
      && position.y < origin.y + size.height
    {
      let work_area = monitor.work_area();
      return Ok(Some((
        display_id,
        work_area.position.to_logical::<f64>(scale),
      )));
    }
  }
  Ok(None)
}

#[cfg(test)]
mod tests {
  use super::*;

  const PLATE: LogicalSize<f64> = LogicalSize {
    width: 836.0,
    height: 48.0,
  };

  /// A 1440x900 screen at x 100 whose top 25 points are the menu bar.
  fn anchor() -> Anchor {
    Anchor {
      display_id: 1,
      origin: LogicalPosition::new(100.0, 25.0),
      size: LogicalSize::new(1_440.0, 875.0),
      screen_origin: LogicalPosition::new(100.0, 0.0),
      screen_size: LogicalSize::new(1_440.0, 900.0),
    }
  }

  fn dropped(display_id: u32, x: f64, y: f64) -> Option<settings::ToolbarPosition> {
    Some(settings::ToolbarPosition { display_id, x, y })
  }

  #[test]
  fn a_toolbar_that_fits_where_it_was_dropped_stays_there() {
    let at = placed(&anchor(), dropped(1, 300.0, 40.0), PLATE);
    assert_eq!((at.x, at.y), (400.0, 65.0));
  }

  #[test]
  fn a_wider_toolbar_near_the_edge_slides_in_rather_than_recentring() {
    let at = placed(&anchor(), dropped(1, 900.0, 40.0), PLATE);
    assert_eq!((at.x, at.y), (100.0 + 1_440.0 - 836.0, 65.0));
  }

  #[test]
  fn a_place_dropped_over_the_menu_bar_is_kept_like_the_recording_bars() {
    let at = placed(&anchor(), dropped(1, 300.0, -20.0), PLATE);
    assert_eq!((at.x, at.y), (400.0, 5.0));
    let above = placed(&anchor(), dropped(1, 300.0, -60.0), PLATE);
    assert_eq!(above.y, 0.0);
  }

  #[test]
  fn a_place_on_another_display_opens_top_centre() {
    let at = placed(&anchor(), dropped(2, 900.0, 40.0), PLATE);
    assert_eq!(
      (at.x, at.y),
      (100.0 + (1_440.0 - 836.0) / 2.0, 25.0 + TOP_GAP)
    );
  }
}
