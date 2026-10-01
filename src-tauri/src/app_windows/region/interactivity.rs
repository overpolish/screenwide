// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Re-asserts that invariant against the window.
///
/// This has to be called by everything that shows the overlay, because
/// `platform::show` turns cursor events back on every time it runs. Leaving it
/// to the caller to remember is what made the desktop stop accepting clicks
/// after a re-show.
pub(super) fn apply_region_selector_interactivity(app: &AppHandle) -> tauri::Result<()> {
  let Some(region) = app.get_webview_window(WindowLabel::RegionSelector.as_str()) else {
    return Ok(());
  };
  let is_interactive = region_selector_is_interactive(
    REGION_SELECTOR_INTERACTIVE.load(Ordering::Relaxed),
    crate::recording::is_idle(app),
  );

  region.set_ignore_cursor_events(!is_interactive)?;
  if !is_interactive {
    // Going passthrough while still holding key status would leave the user
    // typing into an invisible overlay instead of the app they are recording,
    // so the overlay gives keyboard focus back as it stops taking clicks.
    #[cfg(target_os = "macos")]
    platform::release_key_focus(&region)?;
    raise_recording_controls(app)?;
  }

  Ok(())
}

#[tauri::command]
pub fn set_region_selector_passthrough(app: AppHandle, passthrough: bool) -> tauri::Result<()> {
  REGION_SELECTOR_INTERACTIVE.store(!passthrough, Ordering::Relaxed);
  apply_region_selector_interactivity(&app)
}
