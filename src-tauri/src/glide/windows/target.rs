// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The HWND and monitor geometry captured when a Windows session begins.

use windows::Win32::{
  Foundation::{HWND, POINT, RECT},
  Graphics::Gdi::{GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST},
  UI::{
    HiDpi::GetDpiForWindow,
    WindowsAndMessaging::{
      GetWindowRect, IsWindow, IsZoomed, SetWindowPos, ShowWindow, ShowWindowAsync, SWP_NOACTIVATE,
      SWP_NOOWNERZORDER, SWP_NOSIZE, SWP_NOZORDER, SW_MAXIMIZE, SW_MINIMIZE, SW_RESTORE,
    },
  },
};

use super::titlebar;
use crate::glide::{
  core::{corrected_origin, frame_fits, frame_fractions, landing_point, GlideFrame},
  region_rect::{region_gravity, region_rect, PlacedRegion},
};

const WINDOWS_DPI: f64 = 96.0;
const FIT_EPSILON: f64 = 2.0;

#[derive(Clone, Copy)]
pub(super) struct WindowTarget {
  hwnd: isize,
  original: RECT,
  work: RECT,
  dpi: u32,
  was_maximized: bool,
}

pub(super) struct Placement {
  pub actual: GlideFrame,
  pub fits: bool,
}

impl WindowTarget {
  pub fn at(point: POINT) -> Option<(Self, Option<u32>)> {
    let hwnd = titlebar::window_at(point)?;
    let mut original = RECT::default();
    unsafe { GetWindowRect(hwnd, &mut original) }.ok()?;
    let monitor = unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST) };
    if monitor.0.is_null() {
      return None;
    }
    let mut info = MONITORINFO {
      cbSize: std::mem::size_of::<MONITORINFO>() as u32,
      ..Default::default()
    };
    if !unsafe { GetMonitorInfoW(monitor, &mut info) }.as_bool() {
      return None;
    }
    let dpi = unsafe { GetDpiForWindow(hwnd) }.max(WINDOWS_DPI as u32);
    Some((
      Self {
        hwnd: hwnd.0 as isize,
        original,
        work: info.rcWork,
        dpi,
        was_maximized: unsafe { IsZoomed(hwnd) }.as_bool(),
      },
      titlebar::process_id(hwnd),
    ))
  }

  pub fn place(self, region: &PlacedRegion, gap: u32) -> Result<Placement, String> {
    self.ensure_window()?;
    let gap = (f64::from(gap) * f64::from(self.dpi) / WINDOWS_DPI).round() as u32;
    let work = frame(self.work);
    let (origin, size) = region_rect((work.x, work.y), (work.width, work.height), region, gap);
    let destination = GlideFrame {
      x: origin.0,
      y: origin.1,
      width: size.0,
      height: size.1,
    };
    if self.was_maximized {
      unsafe { ShowWindow(self.hwnd(), SW_RESTORE) };
    }
    set_frame(self.hwnd(), destination, false)?;
    let mut achieved = self.frame()?;
    let fits = frame_fits(achieved, destination, FIT_EPSILON);
    if !fits {
      let (x, y) = corrected_origin(destination, achieved, region_gravity(region));
      achieved.x = x;
      achieved.y = y;
      set_frame(self.hwnd(), achieved, true)?;
      achieved = self.frame()?;
    }
    let actual = frame_fractions(achieved, (work.x, work.y), (work.width, work.height))
      .ok_or_else(|| "The Glide monitor has no usable work area".to_owned())?;
    Ok(Placement { actual, fits })
  }

  pub fn restore(self) {
    if self.was_maximized {
      unsafe { ShowWindow(self.hwnd(), SW_MAXIMIZE) };
    } else {
      let _ = set_frame(self.hwnd(), frame(self.original), false);
    }
  }

  pub fn minimize(self) {
    if self.ensure_window().is_ok() {
      let _ = unsafe { ShowWindowAsync(self.hwnd(), SW_MINIMIZE) };
    }
  }

  pub fn landing(self, anchor: POINT) -> POINT {
    let achieved = self.frame().unwrap_or_else(|_| frame(self.original));
    let (x, y) = landing_point(
      (f64::from(anchor.x), f64::from(anchor.y)),
      frame(self.original),
      achieved,
    );
    POINT {
      x: x.round() as i32,
      y: y.round() as i32,
    }
  }

  fn frame(self) -> Result<GlideFrame, String> {
    let mut rect = RECT::default();
    unsafe { GetWindowRect(self.hwnd(), &mut rect) }
      .map_err(|error| format!("Could not read the Glide window frame: {error}"))?;
    Ok(frame(rect))
  }

  fn ensure_window(self) -> Result<(), String> {
    if unsafe { IsWindow(Some(self.hwnd())) }.as_bool() {
      Ok(())
    } else {
      Err("The window being glided is no longer available".to_owned())
    }
  }

  fn hwnd(self) -> HWND {
    HWND(self.hwnd as *mut core::ffi::c_void)
  }
}

fn set_frame(hwnd: HWND, frame: GlideFrame, position_only: bool) -> Result<(), String> {
  let size_flag = if position_only {
    SWP_NOSIZE
  } else {
    Default::default()
  };
  unsafe {
    SetWindowPos(
      hwnd,
      None,
      frame.x.round() as i32,
      frame.y.round() as i32,
      frame.width.round().max(1.0) as i32,
      frame.height.round().max(1.0) as i32,
      SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOZORDER | size_flag,
    )
  }
  .map_err(|error| format!("Could not place the Glide window: {error}"))
}

fn frame(rect: RECT) -> GlideFrame {
  GlideFrame {
    x: f64::from(rect.left),
    y: f64::from(rect.top),
    width: f64::from(rect.right - rect.left),
    height: f64::from(rect.bottom - rect.top),
  }
}
