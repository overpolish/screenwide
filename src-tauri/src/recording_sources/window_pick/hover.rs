// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The window the pick is on, as the overlay and the island are told it.

use std::path::PathBuf;

use serde::Serialize;
use ts_rs::TS;

use super::super::MonitorDetails;
use super::targets::display_frame;
use super::Pick;

/// The window the pick is on, as the overlay cuts it out and the island
/// names it: its frame relative to the pick's anchor display, the one the
/// overlay stays on, in logical points as the overlay draws a region. A
/// window on another display lies past the anchor's bounds, where the
/// overlay's desktop plane carries it.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WindowPickHover {
  app_name: String,
  title: String,
  /// The app's icon, once it has been looked up.
  icon_path: Option<PathBuf>,
  monitor_id: u32,
  x: f64,
  y: f64,
  width: f64,
  height: f64,
}

/// The display holding `point`, in the pointer's space.
pub(super) fn monitor_at(
  monitors: &[MonitorDetails],
  (px, py): (f64, f64),
) -> Option<&MonitorDetails> {
  monitors.iter().find(|monitor| {
    let ((x, y, width, height), _) = display_frame(monitor);
    px >= x && px < x + width && py >= y && py < y + height
  })
}

/// The hovered window as the overlay and island are told it. It is measured
/// on the display holding its middle, or the first when none does, in that
/// display's logical points, then carried to the anchor's: the desktop plane
/// lays each display at its physical origin over its own scale, which is its
/// `position`.
pub(super) fn hover_of(pick: &Pick) -> Option<WindowPickHover> {
  let target = &pick.targets[pick.hovered?];
  let middle = (
    f64::from(target.position.x) + f64::from(target.size.width) / 2.0,
    f64::from(target.position.y) + f64::from(target.size.height) / 2.0,
  );
  let home = monitor_at(&pick.monitors, middle).or_else(|| pick.monitors.first())?;
  let anchor = pick
    .monitors
    .iter()
    .find(|monitor| monitor.id == pick.anchor)
    .unwrap_or(home);
  let ((x, y, _, _), scale) = display_frame(home);
  let shift = (
    f64::from(home.position.x - anchor.position.x),
    f64::from(home.position.y - anchor.position.y),
  );
  Some(WindowPickHover {
    app_name: target.app_name.clone(),
    title: target.title.clone(),
    icon_path: pick.icons.get(&target.pid).cloned().flatten(),
    monitor_id: anchor.id,
    x: (f64::from(target.position.x) - x) / scale + shift.0,
    y: (f64::from(target.position.y) - y) / scale + shift.1,
    width: f64::from(target.size.width) / scale,
    height: f64::from(target.size.height) / scale,
  })
}
