// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use chrono::Local;
use tauri::AppHandle;

use super::{cancel, island};
use crate::screenshots::ScreenshotTarget;

/// Releases Escape and puts the island away, saying the capture is over so it
/// settles rather than spinning as it tucks away.
async fn finish(app: &AppHandle) {
  cancel::disarm(app);
  island::withdraw(app).await;
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

  crate::editor::reserve_screenshot_workspace(&app)?;
  crate::capture_overlays::dismiss_all(&app);
  let _ = crate::app_windows::hide_recording_ui(app.clone());
  // Armed before the island comes out, so it is told from the first whether
  // Escape stops the capture.
  let escape_claimed = cancel::arm(&app);
  island::present(&app, escape_claimed).await;

  // Scrolling and stitching run while the app shows nothing but its island,
  // where it would otherwise be napped.
  let activity = crate::user_work::UserActivity::begin("Capturing a scrolling area");
  let captured = super::capture_canvas(&app, target).await;
  drop(activity);
  let image = match captured {
    Ok(image) => image,
    Err(error) => {
      // Stopping on request is an outcome, not a failure: the caller resolves
      // so the toolbar never flashes an error at someone who asked for this.
      let cancelled = cancel::was_requested();
      finish(&app).await;
      crate::editor::release_screenshot_workspace(&app);
      let _ = crate::app_windows::show_recording_ui(&app);
      return if cancelled { Ok(()) } else { Err(error) };
    }
  };

  if let Err(error) = crate::editor::present_screenshot(
    &app,
    image,
    crate::screenshots::capture_scale(target),
    Vec::new(),
    crate::screenshots::capture_file_stem(Local::now().naive_local()),
  ) {
    finish(&app).await;
    crate::editor::release_screenshot_workspace(&app);
    let _ = crate::app_windows::show_recording_ui(&app);
    return Err(error);
  }

  finish(&app).await;

  Ok(())
}
