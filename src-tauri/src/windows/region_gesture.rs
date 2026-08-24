// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Manager};

use super::{
  hide_recording_options, platform, region, source_selector, WindowLabel,
  RECORDING_CONTROLS_VISIBLE,
};

static ACTIVE: AtomicBool = AtomicBool::new(false);

pub(super) fn is_active() -> bool {
  ACTIVE.load(Ordering::Relaxed)
}

#[tauri::command]
pub fn begin_region_selector_gesture(app: AppHandle) -> tauri::Result<()> {
  if !crate::recording::is_idle(&app) || ACTIVE.swap(true, Ordering::Relaxed) {
    return Ok(());
  }

  let result = (|| {
    hide_recording_options(app.clone())?;
    source_selector::collapse_recording_source_selector(app.clone())?;
    if let Some(bar) = app.get_webview_window(WindowLabel::RecordingBar.as_str()) {
      platform::hide(&bar)?;
    }
    if let Some(selector) = app.get_webview_window(WindowLabel::RecordingSourceSelector.as_str()) {
      platform::hide(&selector)?;
    }
    Ok(())
  })();

  if result.is_err() {
    ACTIVE.store(false, Ordering::Relaxed);
    let _ = restore_recording_controls(&app);
  }
  result
}

fn restore_recording_controls(app: &AppHandle) -> tauri::Result<()> {
  if !RECORDING_CONTROLS_VISIBLE.load(Ordering::Relaxed) || !crate::recording::is_idle(app) {
    return Ok(());
  }

  if let Some(bar) = app.get_webview_window(WindowLabel::RecordingBar.as_str()) {
    platform::show(&bar, 1.0)?;
    platform::restore_recording_level(&bar)?;
  }
  if source_selector::is_visible() && region::source_selector_may_show() {
    source_selector::show(app)?;
  }
  Ok(())
}

#[tauri::command]
pub fn finish_region_selector_gesture(app: AppHandle) -> tauri::Result<()> {
  if !ACTIVE.swap(false, Ordering::Relaxed) {
    return Ok(());
  }
  restore_recording_controls(&app)
}
