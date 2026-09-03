// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Native Ruler's temporary passthrough handoff to Quick Screenshot.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::AppHandle;

use super::ruler_windows;
use crate::capture_overlays;

static ACTIVE: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "windows")]
pub(super) fn is_active() -> bool {
  ACTIVE.load(Ordering::Acquire)
}

pub(super) fn reset() {
  ACTIVE.store(false, Ordering::Release);
}

pub(super) async fn set(app: &AppHandle, active: bool) -> Result<(), String> {
  ACTIVE.store(active, Ordering::Release);
  for window in ruler_windows(app) {
    capture_overlays::set_level(
      &window,
      if active {
        26
      } else {
        capture_overlays::FOREGROUND_LEVEL
      },
    )?;
    super::adapter::set_screenshot_mode(&window, active)?;
  }
  #[cfg(target_os = "windows")]
  if active {
    // Display affinity is committed by DWM. Let that compositor frame land
    // before the shutter, as the Region overlay does for its exclusion.
    tokio::time::sleep(std::time::Duration::from_millis(75)).await;
  }
  Ok(())
}
