// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Native Ruler's temporary passthrough handoff to Quick Screenshot.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::AppHandle;

use super::ruler_windows;
use crate::capture_overlays;

static ACTIVE: AtomicBool = AtomicBool::new(false);

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
    #[cfg(target_os = "macos")]
    super::native_overlay_macos::set_screenshot_mode(&window, active)?;
  }
  Ok(())
}
