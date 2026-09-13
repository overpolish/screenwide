// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[tauri::command]
pub fn hide_recording_ui(app: AppHandle) -> tauri::Result<()> {
  // A Quick Screenshot is driven by the Region Selector webview. Hiding that
  // window during `capture_still` suspends its promise continuation before it
  // can restore the ruler's click handling and clear the screenshot session.
  // The frontend calls this command again after that cleanup is complete.
  if !region::recording_ui_may_hide(region::SCREENSHOT_REGION_SESSION.load(Ordering::Acquire)) {
    return Ok(());
  }

  RECORDING_CONTROLS_VISIBLE.store(false, Ordering::Relaxed);
  source_selector::hide(&app)?;
  hide_recording_bar(&app)?;
  region::hide_region_selector(app.clone())?;

  Ok(())
}

pub fn show_recording_ui(app: &AppHandle) -> tauri::Result<()> {
  crate::capture_overlays::dismiss_all(app);
  if !crate::recording::is_idle(app) {
    return Ok(());
  }

  RECORDING_CONTROLS_VISIBLE.store(true, Ordering::Relaxed);
  let bar = app
    .get_webview_window(WindowLabel::RecordingBar.as_str())
    .ok_or_else(|| tauri::Error::WindowNotFound)?;
  platform::show(&bar, 1.0)?;
  escape::sync(
    app,
    true,
    region::SCREENSHOT_REGION_SESSION.load(Ordering::Relaxed),
    crate::ruler::is_active(app),
  );
  // Asserted rather than assumed: a screenshot session may have borrowed and
  // hidden the bar. Coming back to idle is where its complete presentation is
  // put right.
  platform::restore_recording_level(&bar)?;

  app.emit_to(
    WindowLabel::RecordingBar.as_str(),
    "recording-ui://shown",
    (),
  )
}

pub fn is_recording_ui_visible() -> bool {
  RECORDING_CONTROLS_VISIBLE.load(Ordering::Relaxed)
}

#[tauri::command]
pub fn toggle_recording_ui(app: AppHandle) -> tauri::Result<()> {
  crate::capture_overlays::dismiss_all(&app);
  if !crate::recording::is_idle(&app) || crate::editor::focus_pending_workspace(&app) {
    return Ok(());
  }

  // Tauri's visibility query describes the original webview window and is not
  // authoritative after macOS converts it into an NSPanel.
  if is_recording_ui_visible() {
    return hide_recording_ui(app);
  }

  #[cfg(target_os = "macos")]
  if !crate::permissions::has_required_recording_permissions(&app) {
    return crate::permissions::show_permissions_window(&app);
  }

  show_recording_ui(&app)
}

pub(crate) fn sync_recording_ui_escape(app: &AppHandle, ruler_active: bool) {
  escape::sync(
    app,
    RECORDING_CONTROLS_VISIBLE.load(Ordering::Relaxed),
    region::SCREENSHOT_REGION_SESSION.load(Ordering::Relaxed),
    ruler_active,
  );
}

#[tauri::command]
pub fn recording_ui_visible() -> bool {
  is_recording_ui_visible()
}
