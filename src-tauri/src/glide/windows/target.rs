// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The HWND and monitor geometry captured when a Windows session begins.

use tauri::AppHandle;
use windows::Win32::{
  Foundation::{HWND, POINT, RECT},
  Graphics::{
    Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS},
    Gdi::{GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST},
  },
  System::Threading::{AttachThreadInput, GetCurrentThreadId},
  UI::{
    HiDpi::GetDpiForWindow,
    WindowsAndMessaging::{
      BringWindowToTop, GetForegroundWindow, GetWindowLongPtrW, GetWindowRect,
      GetWindowThreadProcessId, IsWindow, IsZoomed, SetForegroundWindow, SetWindowPos, ShowWindow,
      ShowWindowAsync, GWL_STYLE, SWP_NOACTIVATE, SWP_NOOWNERZORDER, SWP_NOSIZE, SWP_NOZORDER,
      SW_MAXIMIZE, SW_MINIMIZE, SW_RESTORE, WS_THICKFRAME,
    },
  },
};

use super::{titlebar, tween};
use crate::glide::{
  core::{landing_point, GlideFrame},
  region_rect::{region_gravity, region_rect, PlacedRegion, RegionGravity},
};

const WINDOWS_DPI: f64 = 96.0;

/// How far the window's invisible resize border extends past its visible
/// frame on each side. `GetWindowRect` and `SetWindowPos` speak in the outer
/// rectangle; Glide places the visible one, so two windows placed edge to
/// edge really touch and a window at the work area's edge really reaches it.
#[derive(Clone, Copy, Default)]
struct FrameInsets {
  left: f64,
  top: f64,
  right: f64,
  bottom: f64,
}

#[derive(Clone, Copy)]
pub(super) struct WindowTarget {
  hwnd: isize,
  /// The visible frame the session found the window in.
  original: GlideFrame,
  insets: FrameInsets,
  work: RECT,
  dpi: u32,
  resizable: bool,
  was_maximized: bool,
}

/// Where a region places this window: the rectangle asked for, the edge it is
/// pulled towards if the window cannot fill it, and the work area the preview
/// reports against.
pub(super) struct Destination {
  pub frame: GlideFrame,
  pub gravity: RegionGravity,
  pub work: GlideFrame,
}

impl WindowTarget {
  pub fn at(app: &AppHandle, point: POINT) -> Option<(Self, Option<u32>)> {
    let hwnd = titlebar::window_at(app, point)?;
    let mut outer = RECT::default();
    unsafe { GetWindowRect(hwnd, &mut outer) }.ok()?;
    let insets = frame_insets(hwnd, outer);
    let original = visible_frame(frame(outer), insets);
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
        insets,
        work: info.rcWork,
        dpi,
        resizable: unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) } & WS_THICKFRAME.0 as isize != 0,
        was_maximized: unsafe { IsZoomed(hwnd) }.as_bool(),
      },
      titlebar::process_id(hwnd),
    ))
  }

  pub fn destination(self, region: &PlacedRegion, gap: u32) -> Destination {
    let gap = (f64::from(gap) * f64::from(self.dpi) / WINDOWS_DPI).round() as u32;
    let work = frame(self.work);
    let (origin, size) = region_rect((work.x, work.y), (work.width, work.height), region, gap);
    Destination {
      frame: GlideFrame {
        x: origin.0,
        y: origin.1,
        width: size.0,
        height: size.1,
      },
      gravity: region_gravity(region),
      work,
    }
  }

  /// Brings the captured window to the front. A background process may not
  /// take the foreground on its own, so the input thread briefly attaches to
  /// the thread that currently owns it and hands the foreground over.
  pub fn raise(self) {
    if self.ensure_window().is_err() {
      return;
    }
    let hwnd = self.hwnd();
    let foreground = unsafe { GetForegroundWindow() };
    if foreground == hwnd {
      return;
    }
    let current = unsafe { GetCurrentThreadId() };
    let owner = if foreground.0.is_null() {
      0
    } else {
      unsafe { GetWindowThreadProcessId(foreground, None) }
    };
    let attached = owner != 0
      && owner != current
      && unsafe { AttachThreadInput(current, owner, true) }.as_bool();
    let _ = unsafe { BringWindowToTop(hwnd) };
    let _ = unsafe { SetForegroundWindow(hwnd) };
    if attached {
      let _ = unsafe { AttachThreadInput(current, owner, false) };
    }
  }

  /// Puts the window back where the session found it. A maximized window
  /// cannot be tweened back into that state, so it snaps.
  pub fn restore(self) {
    if self.was_maximized {
      tween::cancel();
      let _ = unsafe { ShowWindow(self.hwnd(), SW_MAXIMIZE) };
    } else {
      tween::animate_to(self, self.original, None);
    }
  }

  pub fn minimize(self) {
    tween::cancel();
    if self.ensure_window().is_ok() {
      let _ = unsafe { ShowWindowAsync(self.hwnd(), SW_MINIMIZE) };
    }
  }

  /// Where the cursor lands after a commit: the same grip on the window it had
  /// at the anchor. A window still travelling is judged by its destination.
  pub fn landing(self, anchor: POINT) -> Option<POINT> {
    let achieved = tween::in_flight_destination()
      .or_else(|| self.frame().ok())
      .unwrap_or(self.original);
    if crate::glide::core::frames_match(self.original, achieved, 1.0) {
      return None;
    }
    let (x, y) = landing_point(
      (f64::from(anchor.x), f64::from(anchor.y)),
      self.original,
      achieved,
    );
    Some(POINT {
      x: x.round() as i32,
      y: y.round() as i32,
    })
  }

  /// Readies the window for a tween and reads the frame it starts from. A
  /// maximized window is restored first: it cannot be moved in that state.
  pub fn prepare_for_move(self) -> Result<GlideFrame, String> {
    self.ensure_window()?;
    if unsafe { IsZoomed(self.hwnd()) }.as_bool() {
      let _ = unsafe { ShowWindow(self.hwnd(), SW_RESTORE) };
    }
    self.frame()
  }

  /// The window's visible frame.
  pub fn frame(self) -> Result<GlideFrame, String> {
    let mut rect = RECT::default();
    unsafe { GetWindowRect(self.hwnd(), &mut rect) }
      .map_err(|error| format!("Could not read the Glide window frame: {error}"))?;
    Ok(visible_frame(frame(rect), self.insets))
  }

  /// Places the window so that its visible frame is `frame`.
  pub fn set_frame(self, frame: GlideFrame) -> Result<(), String> {
    set_window_frame(self.hwnd(), outer_frame(frame, self.insets), false)
  }

  pub fn set_origin(self, frame: GlideFrame) -> Result<(), String> {
    set_window_frame(self.hwnd(), outer_frame(frame, self.insets), true)
  }

  pub fn is_resizable(self) -> bool {
    self.resizable
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

fn set_window_frame(hwnd: HWND, frame: GlideFrame, position_only: bool) -> Result<(), String> {
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

/// The invisible border around the visible frame, from the extended frame
/// bounds DWM reports. A window DWM does not describe has none.
fn frame_insets(hwnd: HWND, outer: RECT) -> FrameInsets {
  let mut visible = RECT::default();
  let read = unsafe {
    DwmGetWindowAttribute(
      hwnd,
      DWMWA_EXTENDED_FRAME_BOUNDS,
      std::ptr::from_mut(&mut visible).cast(),
      std::mem::size_of::<RECT>() as u32,
    )
  };
  if read.is_err() {
    return FrameInsets::default();
  }
  FrameInsets {
    left: f64::from(visible.left - outer.left).max(0.0),
    top: f64::from(visible.top - outer.top).max(0.0),
    right: f64::from(outer.right - visible.right).max(0.0),
    bottom: f64::from(outer.bottom - visible.bottom).max(0.0),
  }
}

fn visible_frame(outer: GlideFrame, insets: FrameInsets) -> GlideFrame {
  GlideFrame {
    x: outer.x + insets.left,
    y: outer.y + insets.top,
    width: (outer.width - insets.left - insets.right).max(0.0),
    height: (outer.height - insets.top - insets.bottom).max(0.0),
  }
}

fn outer_frame(visible: GlideFrame, insets: FrameInsets) -> GlideFrame {
  GlideFrame {
    x: visible.x - insets.left,
    y: visible.y - insets.top,
    width: visible.width + insets.left + insets.right,
    height: visible.height + insets.top + insets.bottom,
  }
}

fn frame(rect: RECT) -> GlideFrame {
  GlideFrame {
    x: f64::from(rect.left),
    y: f64::from(rect.top),
    width: f64::from(rect.right - rect.left),
    height: f64::from(rect.bottom - rect.top),
  }
}
