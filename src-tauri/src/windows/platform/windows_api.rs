// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[cfg(target_os = "windows")]
pub fn set_capture_affinity(
  window: &WebviewWindow,
  record_screenwide_windows: bool,
) -> tauri::Result<()> {
  use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{
      GetWindowDisplayAffinity, SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE, WDA_NONE,
    },
  };

  let hwnd = HWND(window.hwnd()?.0);
  let desired_affinity = if record_screenwide_windows {
    WDA_NONE
  } else {
    WDA_EXCLUDEFROMCAPTURE
  };
  unsafe {
    let mut current_affinity = 0;
    GetWindowDisplayAffinity(hwnd, &mut current_affinity).map_err(std::io::Error::other)?;
    if current_affinity != desired_affinity.0 {
      SetWindowDisplayAffinity(hwnd, desired_affinity).map_err(std::io::Error::other)?;
    }
  }

  Ok(())
}

#[cfg(target_os = "windows")]
pub fn is_visible(window: &WebviewWindow) -> tauri::Result<bool> {
  use windows::Win32::{Foundation::HWND, UI::WindowsAndMessaging::IsWindowVisible};

  Ok(unsafe { IsWindowVisible(HWND(window.hwnd()?.0)).as_bool() })
}

#[cfg(target_os = "windows")]
pub fn prepare_to_show(window: &WebviewWindow) -> tauri::Result<()> {
  let record_screenwide_windows =
    crate::settings::current(window.app_handle()).record_screenwide_windows;
  set_capture_affinity(window, record_screenwide_windows)
}

#[cfg(target_os = "windows")]
pub fn initialize_recording_bar(window: &WebviewWindow) -> tauri::Result<()> {
  initialize_overlay(window)?;
  super::round_corners(window)
}

#[cfg(target_os = "windows")]
pub fn initialize_recording_source_selector(window: &WebviewWindow) -> tauri::Result<()> {
  initialize_overlay(window)?;
  super::round_corners(window)
}

#[cfg(target_os = "windows")]
pub fn initialize_region_selector(window: &WebviewWindow) -> tauri::Result<()> {
  initialize_overlay(window)
}

#[cfg(target_os = "windows")]
pub fn initialize_standalone_listbox(window: &WebviewWindow) -> tauri::Result<()> {
  initialize_overlay(window)?;
  super::round_corners(window)
}

#[cfg(target_os = "windows")]
pub fn initialize_recording_dock(window: &WebviewWindow) -> tauri::Result<()> {
  initialize_overlay(window)?;
  super::round_corners(window)
}

/// The tooltip is an overlay like the rest, and never activated: it is shown
/// with `raise_without_activation`, which keeps focus where the user left it.
#[cfg(target_os = "windows")]
pub fn initialize_tooltip(window: &WebviewWindow) -> tauri::Result<()> {
  initialize_overlay(window)?;
  super::round_corners(window)
}

#[cfg(target_os = "windows")]
pub fn initialize_editor(window: &WebviewWindow) -> tauri::Result<()> {
  initialize_capture_affinity(window)
}

#[cfg(target_os = "windows")]
pub fn restore_recording_level(window: &WebviewWindow) -> tauri::Result<()> {
  raise_without_activation(window)
}

/// Puts the window back at the ordinary window level, so it stands with the
/// window it belongs to instead of floating over every other application.
///
/// This reorders through Win32 rather than `set_always_on_top(false)` because
/// `restore_recording_level` raises through Win32 too: tao's tracked
/// always-on-top flag goes stale, and it only reorders on a change of that
/// flag, so the API call would silently do nothing.
#[cfg(target_os = "windows")]
pub fn set_normal_level(window: &WebviewWindow) -> tauri::Result<()> {
  use windows::Win32::UI::WindowsAndMessaging::HWND_NOTOPMOST;

  reorder(window, HWND_NOTOPMOST)
}

#[cfg(target_os = "windows")]
pub fn set_opacity(window: &WebviewWindow, opacity: f64) -> tauri::Result<()> {
  composition::set_opacity(window, opacity)
}

#[cfg(target_os = "windows")]
pub fn raise_without_activation(window: &WebviewWindow) -> tauri::Result<()> {
  use windows::Win32::UI::WindowsAndMessaging::HWND_TOPMOST;

  reorder(window, HWND_TOPMOST)
}

/// Moves the window to one of the z-order bands without disturbing its frame
/// or taking focus from wherever the user left it.
#[cfg(target_os = "windows")]
fn reorder(window: &WebviewWindow, band: windows::Win32::Foundation::HWND) -> tauri::Result<()> {
  use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{SetWindowPos, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE},
  };

  unsafe {
    SetWindowPos(
      HWND(window.hwnd()?.0),
      Some(band),
      0,
      0,
      0,
      0,
      SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
    )
    .map_err(std::io::Error::other)?;
  }
  Ok(())
}

#[cfg(target_os = "windows")]
pub fn hide(window: &WebviewWindow) -> tauri::Result<()> {
  window.hide()
}

#[cfg(target_os = "windows")]
pub fn show(window: &WebviewWindow, opacity: f64) -> tauri::Result<()> {
  set_opacity(window, opacity)?;
  prepare_to_show(window)?;
  composition::refresh(window)?;
  window.show()
}
