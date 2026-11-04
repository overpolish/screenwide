// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keeping the system's bars out of screen recordings: the menu bar and the
//! Dock on macOS, the taskbar on Windows. A screen recording that leaves one
//! out records the rest of the display, as a region recording of it would.
//! A bar set to hide itself takes no room and is left in: it slides over
//! the picture when it shows.

use screenwide_capture::{RecordingMode, Region, StartRecordingOptions};
use tauri::{AppHandle, LogicalPosition, LogicalSize};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

/// Less than this along an edge is no bar: macOS keeps a few points free
/// where a hidden Dock slides out.
const SMALLEST_BAR: f64 = 8.0;

/// How far in from each of a display's edges something reaches, in its
/// points.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Insets {
  pub top: f64,
  pub bottom: f64,
  pub left: f64,
  pub right: f64,
}

/// A display's size and the room its bars take, in its points. The menu bar
/// is the taskbar on Windows, where there is no Dock.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct DisplayBars {
  pub width: f64,
  pub height: f64,
  pub menu_bar: Insets,
  pub dock: Insets,
}

/// What a recording made with `options` keeps of its display: none but for
/// a screen recording with a bar left out.
pub(super) fn recorded_area(app: &AppHandle, options: &StartRecordingOptions) -> Option<Region> {
  if options.mode != RecordingMode::Screen {
    return None;
  }
  screen_area(app, options.monitor_id?)
}

/// What a screen recording of display `monitor_id` keeps of it, as shares
/// of its width and height from its top-left corner (left, top, width,
/// height), for a still of the display to show what is recorded; none to
/// keep the whole display.
pub(crate) fn kept_share(app: &AppHandle, monitor_id: u32) -> Option<[f64; 4]> {
  let (region, bars) = kept(app, monitor_id)?;
  Some([
    region.position.x / bars.width,
    region.position.y / bars.height,
    region.size.width / bars.width,
    region.size.height / bars.height,
  ])
}

/// The part of display `monitor_id` a screen recording keeps, by the bars
/// Settings leaves out; none to keep the whole display.
fn screen_area(app: &AppHandle, monitor_id: u32) -> Option<Region> {
  kept(app, monitor_id).map(|(region, _)| region)
}

fn kept(app: &AppHandle, monitor_id: u32) -> Option<(Region, DisplayBars)> {
  let settings = crate::settings::current(app);
  if !settings.hide_menu_bar && !settings.hide_dock {
    return None;
  }
  #[cfg(target_os = "macos")]
  let bars = macos::display_bars(app, monitor_id)?;
  #[cfg(target_os = "windows")]
  let bars = windows::display_bars(monitor_id)?;
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  let bars: DisplayBars = {
    let _ = (app, monitor_id);
    return None;
  };
  let region = kept_region(&bars, settings.hide_menu_bar, settings.hide_dock)?;
  Some((region, bars))
}

/// The display less the bars chosen, in its points from its top-left
/// corner; none where that leaves it whole.
fn kept_region(bars: &DisplayBars, menu_bar: bool, dock: bool) -> Option<Region> {
  let edge = |reach: f64| if reach >= SMALLEST_BAR { reach } else { 0.0 };
  let mut cut = Insets::default();
  for (insets, chosen) in [(bars.menu_bar, menu_bar), (bars.dock, dock)] {
    if chosen {
      cut.top += edge(insets.top);
      cut.bottom += edge(insets.bottom);
      cut.left += edge(insets.left);
      cut.right += edge(insets.right);
    }
  }
  if cut == Insets::default() {
    return None;
  }
  let width = bars.width - cut.left - cut.right;
  let height = bars.height - cut.top - cut.bottom;
  (width > 0.0 && height > 0.0).then(|| Region {
    position: LogicalPosition::new(cut.left, cut.top),
    size: LogicalSize::new(width, height),
  })
}

#[cfg(test)]
mod tests;
