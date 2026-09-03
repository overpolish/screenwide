// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Manager};

use super::{
  hide_recording_options, platform, source_selector, WindowLabel, RECORDING_CONTROLS_VISIBLE,
};

static ACTIVE: AtomicBool = AtomicBool::new(false);

fn conceal_recording_bar(window: &tauri::WebviewWindow) -> tauri::Result<()> {
  #[cfg(target_os = "windows")]
  {
    // Keep WebView2 presented while the region gesture borrows its space. A
    // hide/show cycle can expose its blank backing surface for one frame when
    // the bar returns.
    window.set_ignore_cursor_events(true)?;
    platform::set_opacity(window, 0.0)
  }
  #[cfg(not(target_os = "windows"))]
  platform::hide(window)
}

fn restore_recording_bar(window: &tauri::WebviewWindow) -> tauri::Result<()> {
  #[cfg(target_os = "windows")]
  {
    platform::prepare_to_show(window)?;
    window.set_ignore_cursor_events(false)?;
    platform::set_opacity(window, 1.0)?;
    platform::restore_recording_level(window)
  }
  #[cfg(not(target_os = "windows"))]
  {
    platform::show(window, 1.0)?;
    platform::restore_recording_level(window)
  }
}

pub(super) fn is_active() -> bool {
  ACTIVE.load(Ordering::Relaxed)
}

#[tauri::command]
pub fn begin_region_selector_gesture(app: AppHandle) -> tauri::Result<()> {
  if !crate::recording::is_idle(&app) || ACTIVE.swap(true, Ordering::Relaxed) {
    return Ok(());
  }

  let result = (|| {
    // Merely showing the editor must not interrupt keyboard navigation in the
    // recording bar. A pointer gesture is the explicit handoff that makes the
    // region editor the keyboard target for editing.
    if let Some(region) = app.get_webview_window(WindowLabel::RegionSelector.as_str()) {
      region.set_focus()?;
    }
    hide_recording_options(app.clone())?;
    source_selector::collapse(app.clone(), Some(false))?;
    if let Some(bar) = app.get_webview_window(WindowLabel::RecordingBar.as_str()) {
      conceal_recording_bar(&bar)?;
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
    restore_recording_bar(&bar)?;
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
