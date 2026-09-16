// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The overlay's handoff to Quick Screenshot.
//!
//! While the region overlay is up the annotations stay on screen but stop
//! taking input, and sit under the region surfaces so the selection draws over
//! them. Drawing resumes when the screenshot session ends, if that is what the
//! overlay was doing when it began.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::AppHandle;

use crate::capture_overlays;

/// Same level the ruler drops to for its own screenshot handoff: under the
/// region overlay, above the desktop.
const SCREENSHOT_LEVEL: isize = 26;

static RESUME_DRAWING: AtomicBool = AtomicBool::new(false);

pub(super) fn set(app: &AppHandle, active: bool) {
  if active {
    let was_drawing = super::is_active(app);
    RESUME_DRAWING.store(was_drawing, Ordering::Release);
    if was_drawing {
      super::stop_drawing(app);
    }
    if let Err(error) = super::host::set_level(app, SCREENSHOT_LEVEL) {
      eprintln!("Could not lower the annotate overlay for a screenshot: {error}");
    }
    return;
  }
  if let Err(error) = super::host::set_level(app, capture_overlays::FOREGROUND_LEVEL) {
    eprintln!("Could not restore the annotate overlay after a screenshot: {error}");
  }
  if RESUME_DRAWING.swap(false, Ordering::AcqRel) {
    // Window creation runs off the thread that services the event loop, the
    // way every other capture overlay opens.
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
      if let Err(error) = super::start(&app) {
        eprintln!("Could not resume the annotate overlay after a screenshot: {error}");
      }
    });
  }
}

/// A dismissal during the screenshot means the user does not want the overlay
/// back afterwards.
pub(super) fn forget_resume() {
  RESUME_DRAWING.store(false, Ordering::Release);
}
