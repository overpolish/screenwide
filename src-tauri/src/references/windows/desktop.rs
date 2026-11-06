// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! The desktop as a reference measures it on Windows. Windows lays its
//! displays out in physical pixels, each at its own scale; a reference
//! measures in points at the scale of the display it stands on, so its
//! picture keeps its size in points there. A window stands on one display
//! at a time, so one scale serves every length it measures.

use screenwide_osc::geometry::{Point, Rect};
use windows::Win32::{
  Foundation::{HWND, POINT, RECT},
  Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromPoint, HMONITOR, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    MONITOR_DEFAULTTOPRIMARY,
  },
  UI::{
    HiDpi::{GetDpiForMonitor, GetDpiForWindow, MDT_EFFECTIVE_DPI},
    WindowsAndMessaging::GetCursorPos,
  },
};

/// Windows reports DPI relative to this baseline.
const BASE_DPI: f64 = 96.0;

fn scale_for(dpi: u32) -> f64 {
  f64::from(dpi.max(BASE_DPI as u32)) / BASE_DPI
}

/// Pixels to the point on the display `window` stands on.
pub(super) fn window_scale(window: HWND) -> f64 {
  scale_for(unsafe { GetDpiForWindow(window) })
}

/// Pixels to the point on the display the physical pixel `at` is on.
pub(in crate::references) fn scale_at(at: POINT) -> f64 {
  let monitor = unsafe { MonitorFromPoint(at, MONITOR_DEFAULTTONEAREST) };
  let (mut dpi, mut unused) = (0, 0);
  match unsafe { GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi, &mut unused) } {
    Ok(()) => scale_for(dpi),
    Err(_) => 1.0,
  }
}

pub(super) fn to_points(rect: RECT, scale: f64) -> Rect {
  Rect::from_xywh(
    f64::from(rect.left) / scale,
    f64::from(rect.top) / scale,
    f64::from(rect.right - rect.left) / scale,
    f64::from(rect.bottom - rect.top) / scale,
  )
}

/// `rect` in physical pixels: its edges each rounded, so lengths laid end
/// to end keep meeting.
pub(super) fn to_pixels(rect: Rect, scale: f64) -> RECT {
  let edge = |points: f64| (points * scale).round() as i32;
  RECT {
    left: edge(rect.origin.x),
    top: edge(rect.origin.y),
    right: edge(rect.origin.x + rect.size.width),
    bottom: edge(rect.origin.y + rect.size.height),
  }
}

pub(super) fn pointer(scale: f64) -> Point {
  let mut at = POINT::default();
  let _ = unsafe { GetCursorPos(&mut at) };
  Point {
    x: f64::from(at.x) / scale,
    y: f64::from(at.y) / scale,
  }
}

fn monitor_info(monitor: HMONITOR) -> Option<MONITORINFO> {
  let mut info = MONITORINFO {
    cbSize: std::mem::size_of::<MONITORINFO>() as u32,
    ..Default::default()
  };
  unsafe { GetMonitorInfoW(monitor, &mut info) }
    .as_bool()
    .then_some(info)
}

/// The display `point` is on, else the nearest.
pub(super) fn display_at(point: Point, scale: f64) -> Option<MONITORINFO> {
  let at = POINT {
    x: (point.x * scale).round() as i32,
    y: (point.y * scale).round() as i32,
  };
  monitor_info(unsafe { MonitorFromPoint(at, MONITOR_DEFAULTTONEAREST) })
}

/// The primary display, whose top left is the desktop's origin.
pub(super) fn primary() -> Option<MONITORINFO> {
  monitor_info(unsafe { MonitorFromPoint(POINT::default(), MONITOR_DEFAULTTOPRIMARY) })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn lengths_laid_end_to_end_keep_meeting_at_a_fractional_scale() {
    let scale = 1.25;
    let left = to_pixels(Rect::from_xywh(10.3, 0.0, 20.3, 5.0), scale);
    let right = to_pixels(Rect::from_xywh(30.6, 0.0, 7.7, 5.0), scale);
    assert_eq!(left.right, right.left);
  }
}
