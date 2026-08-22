// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use chrono::Local;
use tauri::AppHandle;

use super::{cancel, overlay, progress};
use crate::screenshots::ScreenshotTarget;

/// Releases Escape and tears the overlay down, telling it the capture is over
/// so it settles rather than spinning in the moment before the window closes.
fn finish(app: &AppHandle) {
  cancel::disarm(app);
  progress::emit_finished(app);
  overlay::close(app);
}

/// Captures the selected scrollable region into one two-dimensional canvas.
#[tauri::command]
pub async fn capture_scrolling_still(
  app: AppHandle,
  target: ScreenshotTarget,
) -> Result<(), String> {
  if !crate::recording::is_idle(&app) {
    return Err("A screenshot cannot be taken while a recording is active".to_owned());
  }
  if !matches!(target, ScreenshotTarget::Region { .. }) {
    return Err("Scrolling capture requires a region".to_owned());
  }

  crate::exports::reserve_screenshot_workspace(&app)?;
  // After the dismissal, so the overlay this capture owns is not swept away
  // with the capture tools it just closed.
  crate::capture_overlays::dismiss_all(&app);
  let _ = crate::windows::hide_recording_ui(app.clone());
  // Armed before the overlay exists so the window is told, as it loads, whether
  // it can offer a way out.
  let cancellable = cancel::arm(&app);
  if let Err(error) = overlay::show(&app, target, cancellable) {
    cancel::disarm(&app);
    crate::exports::release_screenshot_workspace(&app);
    let _ = crate::windows::show_recording_ui(&app);
    return Err(error);
  }

  let image = match super::capture_canvas(&app, target).await {
    Ok(image) => image,
    Err(error) => {
      // Stopping on request is an outcome, not a failure: the caller resolves
      // so the toolbar never flashes an error at someone who asked for this.
      let cancelled = cancel::was_requested();
      finish(&app);
      crate::exports::release_screenshot_workspace(&app);
      let _ = crate::windows::show_recording_ui(&app);
      return if cancelled { Ok(()) } else { Err(error) };
    }
  };

  if let Err(error) = crate::exports::present_screenshot(
    &app,
    image,
    crate::screenshots::capture_file_stem(Local::now().naive_local()),
  ) {
    finish(&app);
    crate::exports::release_screenshot_workspace(&app);
    let _ = crate::windows::show_recording_ui(&app);
    return Err(error);
  }

  finish(&app);

  Ok(())
}
