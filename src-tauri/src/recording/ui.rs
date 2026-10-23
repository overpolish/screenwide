// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::AppHandle;

use crate::app_windows;
use screenwide_capture::{RecordingMode, StartRecordingOptions};

// ---------------------------------------------------------------------------
// Window choreography. Every call here composes existing window commands and
// runs with no recording lock held.
// ---------------------------------------------------------------------------

pub(super) fn prepare_windows(
  app: &AppHandle,
  options: &StartRecordingOptions,
) -> Result<(), String> {
  let to_message = |error: tauri::Error| error.to_string();

  crate::recording_sources::window_pick::cancel(app);
  app_windows::close_recording_controls(app).map_err(to_message)?;
  // The island shows the recording from here, out already if the controls
  // started it, brought out if a shortcut did. Its Starting state keeps the
  // controls from being used until the first encoded frames prove that every
  // configured video stream is ready.
  app_windows::island::show(app).map_err(to_message)?;
  // A recording is one of our own captures for its whole length. The windows
  // whose exclusion depends on that follow it from here, which gives the
  // compositor the countdown to present the excluded state.
  #[cfg(target_os = "windows")]
  {
    app_windows::mark_capturing(app_windows::Capture::Recording, true);
    app_windows::apply_capture_policy(app).map_err(to_message)?;
  }

  if options.mode == RecordingMode::Region {
    // Retain the desktop-wide cutout surfaces, but remove editing chrome and
    // stop taking clicks now that the user is no longer editing the region.
    app_windows::screenshot_region::presentation::set_region_selector_osc_frame_visible(
      app.clone(),
      false,
    )?;
    app_windows::set_region_selector_passthrough(app.clone(), true).map_err(to_message)?;
  } else {
    app_windows::hide_region_selector(app.clone()).map_err(to_message)?;
  }

  Ok(())
}

/// Puts the recording's windows back. The island is left out: whether it goes
/// depends on where the recording ends up (`hide_island`,
/// `show_recording_ui`).
pub(super) fn restore_windows(app: &AppHandle) {
  if app_windows::is_region_selector_visible(app) {
    // Interactivity is deliberately not touched here. The overlay is about to
    // be hidden, and when the controls show it again `show_region_selector`
    // re-asserts the invariant for us.
    let _ = app_windows::hide_region_selector(app.clone());
  }
  #[cfg(target_os = "windows")]
  {
    app_windows::mark_capturing(app_windows::Capture::Recording, false);
    if let Err(error) = app_windows::apply_capture_policy(app) {
      eprintln!("Could not restore capture affinity after recording: {error}");
    }
  }
}

/// Shows the recording controls again, in the island that showed the
/// recording. Must run after the snapshot is back to `Idle` so their
/// mode-driven UI sync is no longer gated.
pub(super) fn show_recording_ui(app: &AppHandle) {
  if let Err(error) = app_windows::show_recording_ui(app) {
    eprintln!("Could not restore the recording controls: {error}");
  }
}

/// The recording is finished and goes to the Editor, so the island that
/// showed it goes.
pub(super) fn hide_island(app: &AppHandle) {
  if let Err(error) = app_windows::island::hide(app) {
    eprintln!("Could not put the island away: {error}");
  }
}
