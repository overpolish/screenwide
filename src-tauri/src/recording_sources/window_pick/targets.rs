// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What picking reads: the windows that can be picked, the pointer, and the
//! displays, all in the pointer's space.

use super::super::{platform, MonitorDetails};
use screenwide_capture::types::{Position, Size};

/// A window that can be picked, its frame in the pointer's space: points on
/// macOS, pixels on Windows, from the top left of the main display.
#[derive(Clone, Debug)]
pub(super) struct Target {
  pub(super) id: u32,
  pub(super) pid: u32,
  pub(super) app_name: String,
  pub(super) title: String,
  pub(super) position: Position,
  pub(super) size: Size,
}

impl Target {
  pub(super) fn contains(&self, (x, y): (f64, f64)) -> bool {
    let left = f64::from(self.position.x);
    let top = f64::from(self.position.y);
    x >= left
      && x < left + f64::from(self.size.width)
      && y >= top
      && y < top + f64::from(self.size.height)
  }
}

#[cfg(target_os = "macos")]
pub(super) fn pointer() -> Option<(f64, f64)> {
  use core_graphics::{
    event::CGEvent,
    event_source::{CGEventSource, CGEventSourceStateID},
  };
  let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState).ok()?;
  let point = CGEvent::new(source).ok()?.location();
  Some((point.x, point.y))
}

#[cfg(target_os = "windows")]
pub(super) fn pointer() -> Option<(f64, f64)> {
  let mut point = ::windows::Win32::Foundation::POINT::default();
  unsafe { ::windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut point) }.ok()?;
  Some((f64::from(point.x), f64::from(point.y)))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(super) fn pointer() -> Option<(f64, f64)> {
  None
}

/// A display's frame in the pointer's space, and how many of that space's
/// units make one of its logical points.
pub(super) fn display_frame(monitor: &MonitorDetails) -> ((f64, f64, f64, f64), f64) {
  if cfg!(target_os = "windows") {
    (
      (
        f64::from(monitor.physical_position.x),
        f64::from(monitor.physical_position.y),
        f64::from(monitor.physical_size.width),
        f64::from(monitor.physical_size.height),
      ),
      f64::from(monitor.scale_factor),
    )
  } else {
    (
      (
        f64::from(monitor.position.x),
        f64::from(monitor.position.y),
        f64::from(monitor.size.width),
        f64::from(monitor.size.height),
      ),
      1.0,
    )
  }
}

/// The windows that can be recorded, as the old list filtered them: none of
/// Screenwide's, only those the platform lets be captured, and none untitled,
/// empty or minimized.
pub(super) fn read_targets() -> Result<Vec<Target>, String> {
  let current_pid = std::process::id();
  let selectable = platform::selectable_window_ids();
  // Both platforms list windows front to back: CoreGraphics' on-screen list
  // on macOS, and `EnumWindows`' z-order on Windows.
  let windows = xcap::Window::all().map_err(|error| error.to_string())?;
  Ok(
    windows
      .into_iter()
      .filter_map(|window| {
        let id = window.id().ok()?;
        let pid = window.pid().ok()?;
        let title = window.title().ok()?;
        let width = window.width().ok()?;
        let height = window.height().ok()?;
        if pid == current_pid
          || selectable.as_ref().is_some_and(|ids| !ids.contains(&id))
          || title.trim().is_empty()
          || width == 0
          || height == 0
          || window.is_minimized().unwrap_or(true)
        {
          return None;
        }
        Some(Target {
          id,
          pid,
          app_name: window.app_name().ok()?,
          title,
          position: Position {
            x: window.x().ok()?,
            y: window.y().ok()?,
          },
          size: Size { width, height },
        })
      })
      .collect(),
  )
}
