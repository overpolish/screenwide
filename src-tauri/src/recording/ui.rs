// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::AppHandle;

use super::{RecordingMode, StartRecordingOptions};
use crate::windows;

// ---------------------------------------------------------------------------
// Window choreography. Every call here composes existing window commands and
// runs with no recording lock held.
// ---------------------------------------------------------------------------

pub(super) fn prepare_windows(
  app: &AppHandle,
  options: &StartRecordingOptions,
) -> Result<(), String> {
  let to_message = |error: tauri::Error| error.to_string();

  windows::hide_recording_options(app.clone()).map_err(to_message)?;
  windows::source_selector::collapse(app.clone(), Some(false)).map_err(to_message)?;
  windows::source_selector::set_recording_source_selector_visible(app.clone(), false)
    .map_err(to_message)?;
  windows::hide_recording_bar(app).map_err(to_message)?;
  // The dock is useful feedback while hardware streams warm up. Its Starting
  // overlay prevents controls being used until the first encoded frames prove
  // that every configured video stream is ready.
  windows::show_recording_dock(app).map_err(to_message)?;

  if options.mode == RecordingMode::Region {
    // Retain the desktop-wide cutout surfaces, but remove editing chrome and
    // stop taking clicks now that the user is no longer editing the region.
    windows::screenshot_region::presentation::set_region_selector_osc_frame_visible(
      app.clone(),
      false,
    )?;
    windows::set_region_selector_passthrough(app.clone(), true).map_err(to_message)?;
  } else {
    windows::hide_region_selector(app.clone()).map_err(to_message)?;
  }

  Ok(())
}

pub(super) fn restore_windows(app: &AppHandle) {
  let _ = windows::hide_recording_dock(app);
  if windows::is_region_selector_visible(app) {
    // Interactivity is deliberately not touched here. The overlay is about to
    // be hidden, and when the bar shows it again `show_region_selector`
    // re-asserts the invariant for us.
    let _ = windows::hide_region_selector(app.clone());
  }
}

/// Shows the recording bar again. Must run after the snapshot is back to
/// `Idle` so the bar's mode-driven UI sync is no longer gated.
pub(super) fn show_recording_ui(app: &AppHandle) {
  if let Err(error) = windows::show_recording_ui(app) {
    eprintln!("Could not restore the recording bar: {error}");
  }
}
