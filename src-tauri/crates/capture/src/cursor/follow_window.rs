// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pointer of a window recording, kept where it is on the window. The
//! recording follows the window as it is dragged, and keeps the size it
//! started at, a window resized since fitted to it and centred
//! (`desktop_capture::window_pieces`, `fitted_window`). The sidecar maps the
//! pointer into the picture by where the window stood as the recording
//! started, so each position is written where it would be had the window
//! stayed there: on the window as it is, carried to the window as it was.

use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use super::{EventSink, RawCursorEvent};
use crate::desktop_capture::DesktopRect;

/// How long a reading of the window's bounds is used for: about a frame at
/// 240 fps, so a drag is followed without reading them for every event.
const FRESH: Duration = Duration::from_millis(4);

/// `(x, y)` on the window standing at `now`, as it would be on the window
/// standing at `start`, the recording's size, with `now` fitted to it and
/// centred as the recording draws it.
pub(super) fn carried(start: DesktopRect, now: DesktopRect, x: f64, y: f64) -> (f64, f64) {
  if now.width <= 0.0 || now.height <= 0.0 {
    return (x, y);
  }
  let scale = (start.width / now.width).min(start.height / now.height);
  let left = start.x + (start.width - now.width * scale) / 2.0;
  let top = start.y + (start.height - now.height * scale) / 2.0;
  (left + (x - now.x) * scale, top + (y - now.y) * scale)
}

struct Bounds {
  window_id: u32,
  start: DesktopRect,
  latest: DesktopRect,
  read_at: Instant,
}

impl Bounds {
  fn now(&mut self, at: Instant) -> DesktopRect {
    if at.saturating_duration_since(self.read_at) >= FRESH {
      // A window that has closed keeps the bounds it last had.
      if let Some(bounds) = platform::bounds(self.window_id) {
        self.latest = bounds;
      }
      self.read_at = at;
    }
    self.latest
  }
}

/// `sink` with every position carried from the window `window_id` as it
/// stands to where it stood as this started; `sink` as it is where the
/// window cannot be read.
pub(super) fn sink(sink: EventSink, window_id: &str) -> EventSink {
  let Some((window_id, start)) = window_id
    .parse::<u32>()
    .ok()
    .and_then(|id| platform::bounds(id).map(|start| (id, start)))
  else {
    return sink;
  };
  let bounds = Arc::new(Mutex::new(Bounds {
    window_id,
    start,
    latest: start,
    read_at: Instant::now(),
  }));
  Arc::new(move |mut event: RawCursorEvent| {
    let (start, now) = {
      let mut bounds = bounds.lock().unwrap_or_else(PoisonError::into_inner);
      (bounds.start, bounds.now(event.at))
    };
    (event.x, event.y) = carried(start, now, event.x, event.y);
    sink(event)
  })
}

#[cfg(target_os = "macos")]
mod platform {
  use crate::desktop_capture::DesktopRect;

  /// In global points from the top start of the main display, as the
  /// pointer is read.
  pub(super) fn bounds(window_id: u32) -> Option<DesktopRect> {
    crate::platform::window_track::bounds(window_id)
  }
}

#[cfg(target_os = "windows")]
mod platform {
  use windows::Win32::Foundation::{HWND, RECT};
  use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};

  use crate::desktop_capture::DesktopRect;

  /// The window as it is drawn, without its invisible resize border, in
  /// global physical pixels, as the pointer is read.
  pub(super) fn bounds(window_id: u32) -> Option<DesktopRect> {
    let mut rect = RECT::default();
    // SAFETY: `rect` is the size the attribute is written as, and a stale
    // handle only makes the call fail.
    unsafe {
      DwmGetWindowAttribute(
        HWND(window_id as usize as *mut core::ffi::c_void),
        DWMWA_EXTENDED_FRAME_BOUNDS,
        std::ptr::from_mut(&mut rect).cast(),
        size_of::<RECT>() as u32,
      )
    }
    .ok()?;
    Some(DesktopRect {
      x: f64::from(rect.left),
      y: f64::from(rect.top),
      width: f64::from(rect.right - rect.left),
      height: f64::from(rect.bottom - rect.top),
    })
  }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod platform {
  use crate::desktop_capture::DesktopRect;

  pub(super) fn bounds(_window_id: u32) -> Option<DesktopRect> {
    None
  }
}
