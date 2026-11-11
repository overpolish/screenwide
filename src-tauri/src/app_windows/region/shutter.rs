// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Leaving the region overlay out of the still it stands over. On Windows the
//! overlay stays on screen and is left out of captures instead, which DWM
//! commits on a later frame: it is left out as the area starts being drawn
//! ([`begin_screenshot_drawing`]), so the shutter at the end of the drawing
//! waits only for whatever of that frame is still to come.

#[cfg(target_os = "windows")]
use std::sync::{Mutex, PoisonError};
#[cfg(target_os = "windows")]
use std::time::{Duration, Instant};

use tauri::Manager;

#[cfg(target_os = "windows")]
use super::region_selector_capture_affinity;

/// How long DWM takes to commit a change of display affinity.
#[cfg(target_os = "windows")]
const AFFINITY_COMMITS: Duration = Duration::from_millis(75);

/// When the overlay was left out of captures; none while it is in them.
#[cfg(target_os = "windows")]
static LEFT_OUT_SINCE: Mutex<Option<Instant>> = Mutex::new(None);

#[tauri::command]
#[allow(clippy::needless_return)]
pub async fn set_region_selector_opacity(
  window: tauri::WebviewWindow,
  opacity: f64,
) -> Result<(), String> {
  #[cfg(target_os = "windows")]
  {
    let left_out_since = apply_capture_affinity(&window, opacity)?;
    if let Some(since) = left_out_since {
      // Let DWM's frame land before the shutter, without changing anything
      // visible on screen.
      tokio::time::sleep(AFFINITY_COMMITS.saturating_sub(since.elapsed())).await;
    }
    return Ok(());
  }

  #[cfg(target_os = "macos")]
  if opacity <= 0.0 {
    super::super::screenshot_region::set_recording_overlay_desktop_presented(&window, false)
      .map_err(|error| error.to_string())?;
  }
  #[cfg(not(target_os = "windows"))]
  return super::platform::set_opacity(&window, opacity).map_err(|error| error.to_string());
}

/// An area has started being drawn. The overlay is left out of captures now,
/// which shows nothing different on screen; macOS hides it at the shutter
/// instead. When the shot goes to the editor, its window is built during the
/// drawing, so the still opens in it at once.
#[tauri::command]
pub fn begin_screenshot_drawing(
  window: tauri::WebviewWindow,
  prepare_editor: bool,
) -> Result<(), String> {
  #[cfg(target_os = "windows")]
  apply_capture_affinity(&window, 0.0)?;
  if prepare_editor {
    crate::editor::prepare_screenshot_window(window.app_handle());
  }
  Ok(())
}

/// Leaves the overlay out of captures at `opacity` zero and puts it back
/// otherwise, answering since when it has been left out.
#[cfg(target_os = "windows")]
fn apply_capture_affinity(
  window: &tauri::WebviewWindow,
  opacity: f64,
) -> Result<Option<Instant>, String> {
  let affinity = region_selector_capture_affinity(
    opacity,
    crate::settings::current(window.app_handle()).record_screenwide_windows,
  );
  // The shutter is one of our own captures: the windows whose exclusion
  // depends on that, rather than on the preference, follow it here.
  super::super::mark_capturing(super::super::Capture::Still, opacity <= 0.0);
  // The shutter only excludes the borrowed Region overlay. Every other
  // Screenwide window continues to follow the user's capture preference.
  super::super::sync_capture_affinity(window.app_handle(), affinity.other_windows)
    .map_err(|error| error.to_string())?;
  super::super::set_window_capture_affinity(window, affinity.region_selector)
    .map_err(|error| error.to_string())?;
  super::super::screenshot_region::set_recording_overlay_capture_affinity(
    window,
    affinity.region_selector,
  )
  .map_err(|error| error.to_string())?;
  let mut left_out_since = LEFT_OUT_SINCE
    .lock()
    .unwrap_or_else(PoisonError::into_inner);
  if opacity <= 0.0 {
    Ok(Some(*left_out_since.get_or_insert_with(Instant::now)))
  } else {
    *left_out_since = None;
    Ok(None)
  }
}
