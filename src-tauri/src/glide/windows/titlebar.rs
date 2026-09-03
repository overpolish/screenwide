// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{LazyLock, RwLock};

use tauri::{AppHandle, Manager};
use windows::Win32::{
  Foundation::{HWND, POINT, RECT},
  UI::{
    HiDpi::{GetDpiForWindow, GetSystemMetricsForDpi},
    WindowsAndMessaging::{
      GetAncestor, GetClassNameW, GetWindowRect, GetWindowThreadProcessId, WindowFromPoint,
      GA_ROOT, SM_CXPADDEDBORDER, SM_CYCAPTION, SM_CYFRAME,
    },
  },
};

const WINDOWS_DPI: f64 = 96.0;
const FALLBACK_TITLEBAR_HEIGHT: f64 = 48.0;
static OWN_WINDOWS: LazyLock<RwLock<Vec<(isize, bool)>>> =
  LazyLock::new(|| RwLock::new(Vec::new()));

pub(super) fn cache_own_windows(app: &AppHandle) {
  let windows = crate::glide::GLIDABLE_WINDOW_LABELS
    .iter()
    .filter_map(|label| {
      let hwnd = app.get_webview_window(label.as_str())?.hwnd().ok()?;
      Some((hwnd.0 as isize, crate::glide::uses_full_surface(*label)))
    })
    .collect();
  *OWN_WINDOWS
    .write()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = windows;
}

pub(super) fn window_at(app: &AppHandle, point: POINT) -> Option<HWND> {
  window_at_with_own(point, |window| own_window_uses_full_surface(app, window))
}

/// Hook-safe titlebar lookup. Low-level hooks must return promptly, so this
/// path uses HWNDs cached during Glide startup and never asks Tauri/WebView for
/// window state from inside the callback.
pub(super) fn cached_window_at(point: POINT) -> Option<HWND> {
  window_at_with_own(point, cached_own_window_uses_full_surface)
}

fn window_at_with_own(point: POINT, own_window: impl FnOnce(HWND) -> Option<bool>) -> Option<HWND> {
  let hovered = unsafe { WindowFromPoint(point) };
  if hovered.0.is_null() {
    return None;
  }
  let window = unsafe { GetAncestor(hovered, GA_ROOT) };
  if window.0.is_null() {
    return None;
  }
  if is_protected_shell_surface(window) {
    return None;
  }
  let mut process_id = 0;
  unsafe { GetWindowThreadProcessId(window, Some(&mut process_id)) };
  if process_id == std::process::id() && own_window(window)? {
    return Some(window);
  }

  // Chromium, WinUI and other client-drawn windows report HTCLIENT for their
  // visual titlebar, so a non-client hit test cannot be the judge. Match
  // macOS's bounded fallback instead: only the top chrome band qualifies,
  // sized for this window's DPI and never extended into the rest of its
  // content.
  titlebar_band_contains(window, point).then_some(window)
}

fn cached_own_window_uses_full_surface(hwnd: HWND) -> Option<bool> {
  OWN_WINDOWS
    .read()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .iter()
    .find_map(|(window, full_surface)| (*window == hwnd.0 as isize).then_some(*full_surface))
}

fn own_window_uses_full_surface(app: &AppHandle, hwnd: HWND) -> Option<bool> {
  crate::glide::GLIDABLE_WINDOW_LABELS
    .iter()
    .find_map(|label| {
      let matches = app
        .get_webview_window(label.as_str())
        .and_then(|window| window.hwnd().ok())
        .is_some_and(|window| window.0 == hwnd.0);
      matches.then(|| crate::glide::uses_full_surface(*label))
    })
}

/// Explorer implements the taskbars and desktop as ordinary top-level HWNDs.
/// They satisfy the geometric titlebar fallback, but are OS shell surfaces,
/// never user windows that Glide may raise, resize, minimize, or reposition.
fn is_protected_shell_surface(window: HWND) -> bool {
  let mut class = [0_u16; 64];
  let length = unsafe { GetClassNameW(window, &mut class) };
  if length == 0 {
    return false;
  }
  is_protected_shell_class(&String::from_utf16_lossy(&class[..length as usize]))
}

fn is_protected_shell_class(class: &str) -> bool {
  matches!(
    class,
    "Shell_TrayWnd" | "Shell_SecondaryTrayWnd" | "Progman" | "WorkerW"
  )
}

fn titlebar_band_contains(window: HWND, point: POINT) -> bool {
  let mut bounds = RECT::default();
  if unsafe { GetWindowRect(window, &mut bounds) }.is_err() {
    return false;
  }
  let dpi = unsafe { GetDpiForWindow(window) }.max(WINDOWS_DPI as u32);
  let system_height = unsafe {
    GetSystemMetricsForDpi(SM_CYCAPTION, dpi)
      + GetSystemMetricsForDpi(SM_CYFRAME, dpi)
      + GetSystemMetricsForDpi(SM_CXPADDEDBORDER, dpi)
  };
  let fallback_height = (FALLBACK_TITLEBAR_HEIGHT * f64::from(dpi) / WINDOWS_DPI).round() as i32;
  let bottom = bounds
    .top
    .saturating_add(system_height.max(fallback_height))
    .min(bounds.bottom);
  point.x >= bounds.left && point.x < bounds.right && point.y >= bounds.top && point.y < bottom
}

pub(super) fn process_id(window: HWND) -> Option<u32> {
  let mut process_id = 0;
  unsafe { GetWindowThreadProcessId(window, Some(&mut process_id)) };
  (process_id != 0).then_some(process_id)
}

#[cfg(test)]
mod tests {
  use super::is_protected_shell_class;

  #[test]
  fn protects_primary_and_secondary_taskbars() {
    assert!(is_protected_shell_class("Shell_TrayWnd"));
    assert!(is_protected_shell_class("Shell_SecondaryTrayWnd"));
  }

  #[test]
  fn protects_desktop_hosts_without_rejecting_application_windows() {
    assert!(is_protected_shell_class("Progman"));
    assert!(is_protected_shell_class("WorkerW"));
    assert!(!is_protected_shell_class("Chrome_WidgetWin_1"));
  }

  #[test]
  fn only_normal_screenwide_windows_are_glidable() {
    let labels = crate::glide::GLIDABLE_WINDOW_LABELS
      .iter()
      .map(|label| label.as_str())
      .collect::<Vec<_>>();
    assert_eq!(
      labels,
      [
        "settings",
        "recording-bar",
        "recording-dock",
        "export-recording",
        "export-screenshot"
      ]
    );
    assert!(!labels.contains(&"glide"));
    assert!(!labels.contains(&"region-selector"));
    assert!(crate::glide::uses_full_surface(
      crate::windows::WindowLabel::RecordingBar
    ));
    assert!(crate::glide::uses_full_surface(
      crate::windows::WindowLabel::RecordingDock
    ));
    assert!(!crate::glide::uses_full_surface(
      crate::windows::WindowLabel::Settings
    ));
  }
}
