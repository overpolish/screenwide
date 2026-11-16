// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::*;

#[cfg(target_os = "windows")]
pub fn set_capture_affinity(
  window: &WebviewWindow,
  record_screenwide_windows: bool,
) -> tauri::Result<()> {
  use ::windows::Win32::{
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
  use ::windows::Win32::{Foundation::HWND, UI::WindowsAndMessaging::IsWindowVisible};

  Ok(unsafe { IsWindowVisible(HWND(window.hwnd()?.0)).as_bool() })
}

#[cfg(target_os = "windows")]
pub fn prepare_to_show(window: &WebviewWindow) -> tauri::Result<()> {
  let record_screenwide_windows =
    crate::settings::current(window.app_handle()).record_screenwide_windows;
  set_capture_affinity(window, record_screenwide_windows)
}

/// The region overlay, never activated by a click: drawing a screenshot's
/// area would otherwise make it the foreground window, and once it is hidden
/// Windows is left with none, which leaves the system-keys hook unseen until
/// another window is focused, so the next Win+Shift+S opens Snipping Tool.
/// tao is told it cannot take focus before the window is first shown, so its
/// rewrites of the extended style keep `WS_EX_NOACTIVATE`. A region gesture
/// and a window pick, which want the keyboard, activate it themselves.
#[cfg(target_os = "windows")]
pub fn initialize_region_selector(window: &WebviewWindow) -> tauri::Result<()> {
  initialize_overlay(window)?;
  window.set_focusable(false)
}

#[cfg(target_os = "windows")]
pub fn initialize_standalone_listbox(window: &WebviewWindow) -> tauri::Result<()> {
  initialize_overlay(window)?;
  super::window_frame::shape_as_panel(window)
}

/// The editor's island window, laid over the editor's whole content area
/// under its title bar. Square and without a rim: a rounded window gets the
/// light rim Windows 11 draws, which would outline the editor's content.
#[cfg(target_os = "windows")]
pub fn initialize_tool_panel(window: &WebviewWindow) -> tauri::Result<()> {
  initialize_overlay(window)?;
  super::window_frame::remove_border(window)
}

/// The tooltip is an overlay like the rest, and never activated: tao is told
/// it cannot take focus before it is first shown, so its rewrites of the
/// extended style keep `WS_EX_NOACTIVATE`, and it is shown with
/// `show_without_activation`.
#[cfg(target_os = "windows")]
pub fn initialize_tooltip(window: &WebviewWindow) -> tauri::Result<()> {
  initialize_overlay(window)?;
  window.set_focusable(false)?;
  super::window_frame::shape_as_panel(window)
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
/// windows it owns, so a window that outlives its owner, as the island
/// outlives the overlay's hosts, must be released before its owner closes.
#[cfg(target_os = "windows")]
pub fn set_owner(window: &WebviewWindow, owner: Option<&WebviewWindow>) -> tauri::Result<()> {
  use ::windows::Win32::{
    Foundation::{GetLastError, SetLastError, HWND, WIN32_ERROR},
    UI::WindowsAndMessaging::{GetWindow, SetWindowLongPtrW, GWLP_HWNDPARENT, GW_OWNER},
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
    // A zero return with an error set is ambiguous: clearing an owner leaves
    // ERROR_INVALID_WINDOW_HANDLE behind even when it succeeds, so the owner
    // the window ended up with decides.
    let owned_by = GetWindow(hwnd, GW_OWNER).unwrap_or_default().0 as isize;
    if previous == 0 && error.0 != 0 && owned_by != owner {
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
  use ::windows::Win32::UI::WindowsAndMessaging::HWND_NOTOPMOST;

  reorder(window, HWND_NOTOPMOST)
}

#[cfg(target_os = "windows")]
pub fn set_opacity(window: &WebviewWindow, opacity: f64) -> tauri::Result<()> {
  windows::composition::set_opacity(window, opacity)
}

#[cfg(target_os = "windows")]
pub fn raise_without_activation(window: &WebviewWindow) -> tauri::Result<()> {
  use ::windows::Win32::UI::WindowsAndMessaging::HWND_TOPMOST;

  reorder(window, HWND_TOPMOST)
}

/// Moves the window to one of the z-order bands without disturbing its frame
/// or taking focus from wherever the user left it.
#[cfg(target_os = "windows")]
fn reorder(window: &WebviewWindow, band: ::windows::Win32::Foundation::HWND) -> tauri::Result<()> {
  use ::windows::Win32::{
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
  window.hide()?;
  crate::app_windows::on_demand::note_hidden(window);
  Ok(())
}

#[cfg(target_os = "windows")]
pub fn show(window: &WebviewWindow, opacity: f64) -> tauri::Result<()> {
  set_opacity(window, opacity)?;
  prepare_to_show(window)?;
  windows::composition::refresh(window)?;
  window.show()
}

/// Shows `window` where it was placed, leaving the keyboard where it was.
///
/// tao shows a window with `SW_SHOW`, which activates it even with
/// `WS_EX_NOACTIVATE`; its `"focus": false` holds for the first show only. A
/// window shown here has to be hidden with `hide_without_activation` too:
/// tao's own flags still say hidden, so its `hide` changes nothing. For the
/// same reason nothing may change a tao flag while it is shown, or tao's
/// rewrite of the extended style would drop `WS_VISIBLE`.
#[cfg(target_os = "windows")]
pub fn show_without_activation(window: &WebviewWindow) -> tauri::Result<()> {
  use ::windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{ShowWindowAsync, SW_SHOWNOACTIVATE},
  };

  set_opacity(window, 1.0)?;
  prepare_to_show(window)?;
  windows::composition::refresh(window)?;
  let _ = unsafe { ShowWindowAsync(HWND(window.hwnd()?.0), SW_SHOWNOACTIVATE) };
  Ok(())
}

#[cfg(target_os = "windows")]
pub fn hide_without_activation(window: &WebviewWindow) -> tauri::Result<()> {
  use ::windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{ShowWindowAsync, SW_HIDE},
  };

  let _ = unsafe { ShowWindowAsync(HWND(window.hwnd()?.0), SW_HIDE) };
  Ok(())
}

/// Makes `window` the foreground window, for a window shown without
/// activation that now wants the keyboard. tao's `set_focus` would do nothing
/// there: it acts only on a window its own flags say is shown.
#[cfg(target_os = "windows")]
pub fn activate(window: &WebviewWindow) -> tauri::Result<()> {
  use ::windows::Win32::{Foundation::HWND, UI::WindowsAndMessaging::SetForegroundWindow};

  let _ = unsafe { SetForegroundWindow(HWND(window.hwnd()?.0)) };
  Ok(())
}
