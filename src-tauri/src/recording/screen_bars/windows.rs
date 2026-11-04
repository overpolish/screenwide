// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The taskbar of a display, from the work area Windows leaves free of it.
//! Other app bars a display reserves room for are left out with it.

use std::ffi::c_void;

use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, HMONITOR, MONITORINFO};

use super::{DisplayBars, Insets};

/// Display `monitor_id`, a monitor handle as capture names it, in points at
/// the scale its region recordings are cropped with.
pub(super) fn display_bars(monitor_id: u32) -> Option<DisplayBars> {
  let scale = f64::from(
    xcap::Monitor::all()
      .ok()?
      .into_iter()
      .find(|monitor| monitor.id().ok() == Some(monitor_id))?
      .scale_factor()
      .ok()?,
  );
  let mut info = MONITORINFO {
    cbSize: std::mem::size_of::<MONITORINFO>() as u32,
    ..Default::default()
  };
  let monitor = HMONITOR(monitor_id as usize as *mut c_void);
  // SAFETY: `info` is sized as the call requires, and a stale handle only
  // makes the call fail.
  if !unsafe { GetMonitorInfoW(monitor, &mut info) }.as_bool() || scale <= 0.0 {
    return None;
  }
  let (whole, work) = (info.rcMonitor, info.rcWork);
  let points = |pixels: i32| f64::from(pixels) / scale;
  Some(DisplayBars {
    width: points(whole.right - whole.left),
    height: points(whole.bottom - whole.top),
    menu_bar: Insets {
      top: points(work.top - whole.top),
      bottom: points(whole.bottom - work.bottom),
      left: points(work.left - whole.left),
      right: points(whole.right - work.right),
    },
    dock: Insets::default(),
  })
}
