// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The region OSC's Windows host: the anchor child and the `WS_POPUP` peers
//! as DirectComposition windows, their window procedure, Win32 timers and
//! cursors, and GDI text. Everything the overlay decides lives in
//! `region_osc`.

mod keyboard;
mod raster;
mod window;
mod window_proc;

use std::ffi::c_void;
use std::sync::OnceLock;

use tauri::WebviewWindow;
use windows::{
  core::{w, PCWSTR},
  Win32::{
    Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, POINT, RECT, WPARAM},
    Graphics::Gdi::ScreenToClient,
    System::LibraryLoader::GetModuleHandleW,
    UI::{
      HiDpi::GetDpiForWindow,
      Input::KeyboardAndMouse::{ReleaseCapture, SetFocus},
      WindowsAndMessaging::{
        CreateWindowExW, DestroyWindow, GetClientRect, GetCursorPos, GetWindowLongPtrW,
        GetWindowRect, KillTimer, LoadCursorW, PostMessageW, SetCursor, SetLayeredWindowAttributes,
        SetTimer, SetWindowLongPtrW, SetWindowPos, ShowWindowAsync, CS_DBLCLKS, GWL_EXSTYLE, HMENU,
        HWND_TOP, HWND_TOPMOST, IDC_ARROW, IDC_CROSS, IDC_HAND, IDC_IBEAM, IDC_SIZEALL,
        IDC_SIZENESW, IDC_SIZENS, IDC_SIZENWSE, IDC_SIZEWE, LWA_ALPHA, SWP_ASYNCWINDOWPOS,
        SWP_NOACTIVATE, SWP_NOOWNERZORDER, SW_HIDE, SW_SHOWNOACTIVATE, WM_APP, WS_EX_LAYERED,
        WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_POPUP,
      },
    },
  },
};

use super::region_osc::host::{
  KeyboardOwner, RegionDrawable, RegionHost, ScreenPoint, TextRaster, WindowId,
};
use super::region_osc::input::CursorShape;
use crate::app_windows::overlay_surface::{self, disable_transitions, light_mode, GpuSurface};
use screenwide_osc::geometry::Rect;

/// Windows reports DPI relative to this baseline.
const BASE_DPI: f64 = 96.0;
/// Asks the overlay to set its cursor again on its own thread's next turn.
const CURSOR_REFRESH_EVENT: u32 = WM_APP + 0x342;

static OVERLAY_CLASS: OnceLock<u16> = OnceLock::new();
static PEER_CLASS: OnceLock<u16> = OnceLock::new();

pub(crate) struct WindowsRegionHost;

fn hwnd(window: WindowId) -> HWND {
  HWND(window.0 as *mut c_void)
}

fn window_id(hwnd: HWND) -> WindowId {
  WindowId(hwnd.0 as isize)
}

fn rect_of(rect: RECT) -> Rect {
  Rect::from_xywh(
    f64::from(rect.left),
    f64::from(rect.top),
    f64::from(rect.right - rect.left),
    f64::from(rect.bottom - rect.top),
  )
}

impl RegionDrawable for GpuSurface {
  fn resize(&mut self, gpu: &screenwide_gpu::Gpu, size: (u32, u32)) {
    GpuSurface::resize(self, gpu, size);
  }

  fn acquire(&self, gpu: &screenwide_gpu::Gpu) -> Result<screenwide_gpu::surface::Frame, String> {
    GpuSurface::acquire(self, gpu)
  }
}

impl RegionHost for WindowsRegionHost {
  fn webview_window(&self, window: &WebviewWindow) -> Option<WindowId> {
    // Tauri hands back its own `windows` binding's handle; the value is the
    // same process-wide token.
    window.hwnd().ok().map(|hwnd| WindowId(hwnd.0 as isize))
  }

  fn create_window(
    &self,
    webview: &WebviewWindow,
    host: WindowId,
    peer: Option<(Rect, bool)>,
  ) -> Result<WindowId, String> {
    window::create_on_owning_thread(webview, hwnd(host), peer).map(window_id)
  }

  fn create_drawable(
    &self,
    gpu: &screenwide_gpu::Gpu,
    window: WindowId,
  ) -> Result<Box<dyn RegionDrawable>, String> {
    Ok(Box::new(GpuSurface::new(gpu, hwnd(window))?))
  }

  fn destroy_window(&self, window: WindowId) {
    let _ = unsafe { DestroyWindow(hwnd(window)) };
  }

  fn set_timer(&self, window: WindowId, timer: usize, interval_ms: u32) {
    let _ = unsafe { SetTimer(Some(hwnd(window)), timer, interval_ms, None) };
  }

  fn kill_timer(&self, window: WindowId, timer: usize) {
    let _ = unsafe { KillTimer(Some(hwnd(window)), timer) };
  }

  fn refresh_cursor_later(&self, window: WindowId) {
    let _ = unsafe {
      PostMessageW(
        Some(hwnd(window)),
        CURSOR_REFRESH_EVENT,
        WPARAM(0),
        LPARAM(0),
      )
    };
  }

  fn focus(&self, window: WindowId) -> bool {
    unsafe { SetFocus(Some(hwnd(window))) }.is_ok()
  }

  fn set_cursor(&self, shape: CursorShape) {
    let name = match shape {
      CursorShape::None => return,
      CursorShape::Crosshair => IDC_CROSS,
      // Windows has no open or closed hand: the drag cursor is the move one.
      CursorShape::Move => IDC_SIZEALL,
      CursorShape::ResizeHorizontal => IDC_SIZEWE,
      CursorShape::ResizeVertical => IDC_SIZENS,
      CursorShape::ResizeNesw => IDC_SIZENESW,
      CursorShape::ResizeNwse => IDC_SIZENWSE,
      CursorShape::Arrow => IDC_ARROW,
      CursorShape::IBeam => IDC_IBEAM,
      CursorShape::Hand => IDC_HAND,
    };
    if let Ok(cursor) = unsafe { LoadCursorW(None, name) } {
      unsafe { SetCursor(Some(cursor)) };
    }
  }

  fn release_pointer(&self) {
    let _ = unsafe { ReleaseCapture() };
  }

  fn cursor_position(&self) -> Option<ScreenPoint> {
    let mut point = POINT::default();
    unsafe { GetCursorPos(&mut point) }
      .ok()
      .map(|()| ScreenPoint {
        x: point.x,
        y: point.y,
      })
  }

  fn light_mode(&self) -> bool {
    light_mode()
  }

  fn show(&self, window: WindowId, visible: bool) {
    let command = if visible { SW_SHOWNOACTIVATE } else { SW_HIDE };
    let _ = unsafe { ShowWindowAsync(hwnd(window), command) };
  }

  /// `orderFrontRegardless` at the parent's level: each peer covers its own
  /// monitor above everything else.
  fn place_peer(&self, window: WindowId, bounds: Rect) {
    let _ = unsafe {
      SetWindowPos(
        hwnd(window),
        Some(HWND_TOPMOST),
        bounds.origin.x as i32,
        bounds.origin.y as i32,
        (bounds.size.width as i32).max(1),
        (bounds.size.height as i32).max(1),
        SWP_ASYNCWINDOWPOS | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
      )
    };
  }

  /// The anchor child stays sized to its host's client area and above the
  /// WebView2 sibling.
  fn raise(&self, window: WindowId, (width, height): (u32, u32)) {
    let _ = unsafe {
      SetWindowPos(
        hwnd(window),
        Some(HWND_TOP),
        0,
        0,
        width.max(1) as i32,
        height.max(1) as i32,
        SWP_ASYNCWINDOWPOS | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
      )
    };
  }

  fn scale(&self, window: WindowId) -> f64 {
    let dpi = unsafe { GetDpiForWindow(hwnd(window)) };
    if dpi == 0 {
      1.0
    } else {
      f64::from(dpi) / BASE_DPI
    }
  }

  fn client_size(&self, window: WindowId) -> (u32, u32) {
    let mut rect = RECT::default();
    if unsafe { GetClientRect(hwnd(window), &mut rect) }.is_err() {
      return (0, 0);
    }
    (
      (rect.right - rect.left).max(0) as u32,
      (rect.bottom - rect.top).max(0) as u32,
    )
  }

  fn screen_rect(&self, window: WindowId) -> Option<Rect> {
    let mut rect = RECT::default();
    unsafe { GetWindowRect(hwnd(window), &mut rect) }.ok()?;
    Some(rect_of(rect))
  }

  fn screen_to_client(&self, window: WindowId, point: ScreenPoint) -> (f64, f64) {
    let mut point = POINT {
      x: point.x,
      y: point.y,
    };
    let _ = unsafe { ScreenToClient(hwnd(window), &mut point) };
    (f64::from(point.x), f64::from(point.y))
  }

  fn set_pointer_passthrough(&self, window: WindowId, is_root: bool, passthrough: bool) -> bool {
    window::set_pointer_passthrough(hwnd(window), is_root, passthrough)
  }

  fn set_capture_affinity(&self, window: WindowId, capturable: bool) -> bool {
    overlay_surface::set_capture_affinity(hwnd(window), capturable)
      .inspect_err(|error| {
        eprintln!("The Windows region OSC peer could not set capture affinity: {error}")
      })
      .is_ok()
  }

  fn text_context(
    &self,
    font_size: f64,
    raster_scale: f64,
    mono: bool,
  ) -> Option<Box<dyn TextRaster>> {
    raster::GdiText::new(font_size, raster_scale, mono).map(|text| Box::new(text) as _)
  }

  fn capture_keyboard(&self, target: WindowId, owner: KeyboardOwner) -> Result<(), String> {
    crate::osc::keyboard_windows::start(target.0, keyboard::overlay(owner))
  }

  fn release_keyboard(&self, owner: KeyboardOwner) {
    crate::osc::keyboard_windows::stop(keyboard::overlay(owner));
  }

  /// The overlay owns `WM_SETCURSOR` while it is hit-testable, so nothing
  /// has to be acquired.
  fn acquire_quick_screenshot_cursor(&self, _app: &tauri::AppHandle) -> Result<(), String> {
    Ok(())
  }

  fn release_quick_screenshot_cursor(&self, _app: &tauri::AppHandle) -> Result<(), String> {
    Ok(())
  }

  fn on_owning_thread(&self, window: &WebviewWindow) -> bool {
    use windows::Win32::{
      System::Threading::GetCurrentThreadId, UI::WindowsAndMessaging::GetWindowThreadProcessId,
    };
    window.hwnd().is_ok_and(|handle| unsafe {
      GetWindowThreadProcessId(HWND(handle.0), None) == GetCurrentThreadId()
    })
  }
}
