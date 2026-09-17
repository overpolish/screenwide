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

/// Lets presses through a window to whatever is underneath, or takes them
/// back.
///
/// The extended style is edited in place rather than through
/// `set_ignore_cursor_events`, which rebuilds the whole word from tao's own
/// flags: that drops the always-on-top band the window was holding, and it is
/// applied through the event loop, so a raise issued right after it can land
/// before the rebuild and be undone. Layering is added but never removed,
/// because a transparent window's own alpha depends on it.
#[cfg(target_os = "windows")]
pub fn set_pointer_passthrough(window: &WebviewWindow, passthrough: bool) -> tauri::Result<()> {
  use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE},
  };

  let hwnd = HWND(window.hwnd()?.0);
  let current = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };
  let next = passthrough_style(current, passthrough);
  if next != current {
    unsafe { SetWindowLongPtrW(hwnd, GWL_EXSTYLE, next) };
  }
  Ok(())
}

#[cfg(target_os = "windows")]
const fn passthrough_style(style: isize, passthrough: bool) -> isize {
  use windows::Win32::UI::WindowsAndMessaging::{WS_EX_LAYERED, WS_EX_TRANSPARENT};

  let transparent = WS_EX_TRANSPARENT.0 as isize;
  if passthrough {
    style | transparent | WS_EX_LAYERED.0 as isize
  } else {
    style & !transparent
  }
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
  use super::passthrough_style;
  use windows::Win32::UI::WindowsAndMessaging::{WS_EX_LAYERED, WS_EX_TOPMOST, WS_EX_TRANSPARENT};

  #[test]
  fn passthrough_keeps_every_unrelated_style_including_the_topmost_band() {
    let base = 0x0012_3456_isize | WS_EX_TOPMOST.0 as isize;
    let through = passthrough_style(base, true);
    assert_ne!(through & WS_EX_TRANSPARENT.0 as isize, 0);
    assert_ne!(through & WS_EX_LAYERED.0 as isize, 0);
    assert_ne!(through & WS_EX_TOPMOST.0 as isize, 0);
    // Taking presses back leaves the layering a transparent window needs.
    let back = passthrough_style(through, false);
    assert_eq!(back & WS_EX_TRANSPARENT.0 as isize, 0);
    assert_ne!(back & WS_EX_LAYERED.0 as isize, 0);
    assert_ne!(back & WS_EX_TOPMOST.0 as isize, 0);
  }
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

/// The annotate toolbar: an overlay like the rest, and never activated.
/// `WS_EX_NOACTIVATE` is what keeps a press on its controls from taking the
/// foreground - and the keyboard - off the anchor host, so the keys that draw
/// keep working while the toolbar is used. WebView2 still receives the press:
/// the style refuses activation, not input.
#[cfg(target_os = "windows")]
pub fn initialize_annotate_toolbar(window: &WebviewWindow) -> tauri::Result<()> {
  use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{
      GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_NOACTIVATE,
    },
  };

  initialize_overlay(window)?;
  super::round_corners(window)?;
  let hwnd = HWND(window.hwnd()?.0);
  let current = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };
  let next = current | WS_EX_NOACTIVATE.0 as isize;
  if next != current {
    unsafe { SetWindowLongPtrW(hwnd, GWL_EXSTYLE, next) };
  }
  Ok(())
}

/// Makes `window` an owned window of `owner`, or ownerless when `owner` is
/// `None`.
///
/// Windows has no window levels: two always-on-top windows are ordered by
/// whichever was raised last, so a panel cannot be kept above a full-screen
/// overlay by asking for a higher level. Ownership is the mechanism that
/// does it - an owned top-level window always stands above its owner - and is
/// the same one the export sheet uses over the editor.
///
/// The owner has to outlive the ownership: destroying a window destroys the
/// windows it owns, so a panel that is kept between sessions must be released
/// before its owner closes.
#[cfg(target_os = "windows")]
pub fn set_owner(window: &WebviewWindow, owner: Option<&WebviewWindow>) -> tauri::Result<()> {
  use windows::Win32::{
    Foundation::{GetLastError, SetLastError, HWND, WIN32_ERROR},
    UI::WindowsAndMessaging::{SetWindowLongPtrW, GWLP_HWNDPARENT},
  };

  let hwnd = HWND(window.hwnd()?.0);
  let owner = match owner {
    Some(owner) => HWND(owner.hwnd()?.0).0 as isize,
    None => 0,
  };
  unsafe {
    SetLastError(WIN32_ERROR(0));
    let previous = SetWindowLongPtrW(hwnd, GWLP_HWNDPARENT, owner);
    let error = GetLastError();
    if previous == 0 && error.0 != 0 {
      return Err(std::io::Error::from_raw_os_error(error.0 as i32).into());
    }
  }
  Ok(())
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
