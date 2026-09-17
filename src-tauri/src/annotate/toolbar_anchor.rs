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

/// The display the toolbar opens on, which is the overlay's anchor host. Its
/// usable area rather than its whole frame: the toolbar has to clear the menu
/// bar and the notch. Kept from the build so a later fit can place the window
/// again without reading the monitor layout a second time.
pub(super) struct Anchor {
  pub(super) display_id: u32,
  pub(super) origin: LogicalPosition<f64>,
  pub(super) size: LogicalSize<f64>,
}

static ANCHOR: Mutex<Option<Anchor>> = Mutex::new(None);

pub(super) fn current() -> MutexGuard<'static, Option<Anchor>> {
  ANCHOR
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Where a toolbar of `size` belongs on the anchor display: where it was last
/// dropped there, or top-centre. A remembered place the toolbar no longer fits
/// in is not a place - a smaller display, or a wider toolbar, takes it back to
/// the middle rather than off the edge.
pub(super) fn placement(anchor: &Anchor, size: LogicalSize<f64>) -> LogicalPosition<f64> {
  let remembered = settings::current()
    .toolbar_position
    .filter(|position| position.display_id == anchor.display_id)
    .filter(|position| {
      position.x >= 0.0
        && position.y >= 0.0
        && position.x + size.width <= anchor.size.width
        && position.y + size.height <= anchor.size.height
    });
  match remembered {
    Some(position) => {
      LogicalPosition::new(anchor.origin.x + position.x, anchor.origin.y + position.y)
    }
    None => LogicalPosition::new(
      anchor.origin.x + ((anchor.size.width - size.width) / 2.0).max(0.0),
      anchor.origin.y + TOP_GAP,
    ),
  }
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
