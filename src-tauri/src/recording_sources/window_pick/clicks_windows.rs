// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! On Windows the overlay's own surfaces take a press while picking and pick
//! the window under it. With their input off for the pick they would let it
//! through: the desktop peers to the window under them, and the anchor
//! surface to the page beneath it, whose web view takes its input in another
//! process and so never hears it. The island sits above the overlay, so its
//! presses go on as usual.

use std::sync::Arc;

use tauri::{AppHandle, Manager};

use crate::app_windows::screenshot_region::region_osc;
use crate::app_windows::WindowLabel;

pub(super) fn catch(app: &AppHandle) {
  let Some(overlay) = app.get_webview_window(WindowLabel::RegionSelector.as_str()) else {
    return;
  };
  let app = app.clone();
  region_osc::take_presses(
    &overlay,
    Some(Arc::new(move || {
      let app = app.clone();
      // Off the window procedure: ending the pick emits and takes locks.
      std::thread::spawn(move || super::pick_hovered(&app));
    })),
  );
}

pub(super) fn release(app: &AppHandle) {
  if let Some(overlay) = app.get_webview_window(WindowLabel::RegionSelector.as_str()) {
    region_osc::take_presses(&overlay, None);
  }
}
