// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Manager};

use super::{island, WindowLabel, RECORDING_CONTROLS_VISIBLE};

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
    // Merely showing the editor must not interrupt keyboard navigation in the
    // recording controls. A pointer gesture is the explicit handoff that makes the
    // region editor the keyboard target for editing.
    if let Some(region) = app.get_webview_window(WindowLabel::RegionSelector.as_str()) {
      region.set_focus()?;
    }
    // Tucked away while the region is dragged out, and taking no clicks as
    // it goes, so a drag across the top of the screen is the region's.
    island::hide(&app)?;
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

  island::show(app)
}

#[tauri::command]
pub fn finish_region_selector_gesture(app: AppHandle) -> tauri::Result<()> {
  if !ACTIVE.swap(false, Ordering::Relaxed) {
    return Ok(());
  }
  restore_recording_controls(&app)
}
