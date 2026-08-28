// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The ruler's temporary handoff to Quick Screenshot.

use std::sync::{
  atomic::{AtomicBool, AtomicU64, Ordering},
  Mutex,
};
use std::time::Duration;

use tauri::{AppHandle, Emitter};

use super::{ruler_windows, set_system_ruler_cursor_visible};
use crate::capture_overlays;

const EVENT: &str = "ruler://screenshot-mode";
const FOCUS_HANDOFF: Duration = Duration::from_millis(200);

/// The region editor takes focus above a deliberately preserved ruler, so the
/// blur it causes must not tear the session down.
static ACTIVE: AtomicBool = AtomicBool::new(false);
static GENERATION: AtomicU64 = AtomicU64::new(0);
static FOCUS_BEFORE_SCREENSHOT: Mutex<Option<String>> = Mutex::new(None);

pub(super) fn is_active() -> bool {
  ACTIVE.load(Ordering::Relaxed)
}

pub(super) fn reset() {
  GENERATION.fetch_add(1, Ordering::Relaxed);
  ACTIVE.store(false, Ordering::Relaxed);
  *FOCUS_BEFORE_SCREENSHOT
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
}

async fn restore_focus(windows: &[tauri::WebviewWindow], generation: u64) -> Result<(), String> {
  // The export window is focused while the still is handed off and may finish
  // that native activation just after its show call returns. Keep the ruler's
  // focus-loss teardown suspended until that handoff has settled, then make
  // the ruler the explicit owner again.
  tokio::time::sleep(FOCUS_HANDOFF).await;
  if GENERATION.load(Ordering::Relaxed) != generation {
    return Ok(());
  }
  let remembered = FOCUS_BEFORE_SCREENSHOT
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .take();
  if let Some(window) = remembered
    .as_deref()
    .and_then(|label| windows.iter().find(|window| window.label() == label))
    .or_else(|| windows.first())
  {
    window.set_focus().map_err(|error| error.to_string())?;
  }
  ACTIVE.store(false, Ordering::Relaxed);
  Ok(())
}

pub(super) async fn set(app: &AppHandle, active: bool) -> Result<(), String> {
  let generation = GENERATION.fetch_add(1, Ordering::Relaxed) + 1;
  let windows = ruler_windows(app);
  let was_active = is_active();
  if active {
    ACTIVE.store(true, Ordering::Relaxed);
    // The macOS ruler hides the system cursor globally so it is invisible on
    // the very first frame. Release that hide before the screenshot selector
    // opens; the ruler webviews will reapply their appropriate state when this
    // mode ends.
    set_system_ruler_cursor_visible(app, true)?;
  }
  if active && !was_active {
    *FOCUS_BEFORE_SCREENSHOT
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner()) = windows
      .iter()
      .find(|window| window.is_focused().unwrap_or(false))
      .map(|window| window.label().to_owned());
  }

  for window in &windows {
    // The region editor must sit above the ruler while the shot is framed;
    // the ruler remains visible underneath and returns to its normal level.
    capture_overlays::set_level(
      window,
      if active {
        26
      } else {
        capture_overlays::FOREGROUND_LEVEL
      },
    )?;
    window
      .emit(EVENT, active)
      .map_err(|error| error.to_string())?;
    window
      .set_ignore_cursor_events(active)
      .map_err(|error| error.to_string())?;
  }

  if active {
    Ok(())
  } else {
    restore_focus(&windows, generation).await
  }
}
