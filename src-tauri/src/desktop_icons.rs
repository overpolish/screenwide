// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The desktop's icons kept out of a recording on Windows. A capture there
//! cannot leave out another program's window, so Explorer's icon view is
//! hidden on screen while a recording of the screen or a region is under
//! way, and shown again when it ends. Only a view this hid is shown again,
//! so a desktop whose icons are turned off stays as it was. macOS leaves the
//! icons out of the capture instead (`capture_kit::capture_exclusion`), with
//! nothing hidden on screen.

use std::sync::{Mutex, PoisonError};

use screenwide_capture::types::{RecordingMode, RecordingSnapshot, RecordingStatus};
use windows::core::{w, BOOL, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
  EnumWindows, FindWindowExW, FindWindowW, IsWindow, IsWindowVisible, ShowWindow, SW_HIDE,
  SW_SHOWNA,
};

/// The icon view this hid, by handle, while it is hidden.
static HIDDEN: Mutex<Option<isize>> = Mutex::new(None);

/// Hides the icons as a recording of the screen or a region starts, the
/// countdown included, and shows them again once it has stopped.
pub(crate) fn follow_recording(snapshot: &RecordingSnapshot) {
  let wanted = snapshot.status != RecordingStatus::Idle
    && matches!(
      snapshot.mode,
      Some(RecordingMode::Screen | RecordingMode::Region)
    )
    && screenwide_capture::chosen_exclusions::current().desktop;
  let mut hidden = HIDDEN.lock().unwrap_or_else(PoisonError::into_inner);
  match (wanted, *hidden) {
    (true, None) => {
      let Some(view) = icon_view() else {
        return;
      };
      if unsafe { IsWindowVisible(view) }.as_bool() {
        let _ = unsafe { ShowWindow(view, SW_HIDE) };
        *hidden = Some(view.0 as isize);
      }
    }
    (false, Some(handle)) => {
      let view = HWND(handle as *mut _);
      // Explorer restarted meanwhile makes a new view, already shown.
      if unsafe { IsWindow(Some(view)) }.as_bool() {
        let _ = unsafe { ShowWindow(view, SW_SHOWNA) };
      }
      *hidden = None;
    }
    _ => {}
  }
}

/// Explorer's view of the desktop's icons: the list inside the shell view,
/// which sits in Progman, or in a WorkerW once a wallpaper slideshow or an
/// animated wallpaper has moved it there.
fn icon_view() -> Option<HWND> {
  let shell = shell_view()?;
  unsafe { FindWindowExW(Some(shell), None, w!("SysListView32"), PCWSTR::null()) }.ok()
}

fn shell_view() -> Option<HWND> {
  unsafe {
    let progman = FindWindowW(w!("Progman"), PCWSTR::null()).ok()?;
    if let Ok(view) = FindWindowExW(Some(progman), None, w!("SHELLDLL_DefView"), PCWSTR::null()) {
      return Some(view);
    }
    let mut found = HWND::default();
    let _ = EnumWindows(Some(find_in_worker), LPARAM(&raw mut found as isize));
    (!found.is_invalid()).then_some(found)
  }
}

unsafe extern "system" fn find_in_worker(window: HWND, found: LPARAM) -> BOOL {
  match unsafe { FindWindowExW(Some(window), None, w!("SHELLDLL_DefView"), PCWSTR::null()) } {
    Ok(view) => {
      unsafe { *(found.0 as *mut HWND) = view };
      false.into()
    }
    Err(_) => true.into(),
  }
}
