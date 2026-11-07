// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::{AppHandle, Manager};

/// Leaves the recording Region shade/cutout visible while temporarily hiding
/// its interactive OSC during a still or scrolling capture.
#[tauri::command]
pub fn set_region_selector_osc_frame_visible(
  app: AppHandle,
  visible: bool,
) -> Result<bool, String> {
  // An overlay not built has no frame to show or hide; one built later draws
  // its frame from the recording's state.
  let Some(target) = app.get_webview_window(super::super::WindowLabel::RegionSelector.as_str())
  else {
    return Ok(false);
  };
  let show = visible && crate::recording::is_idle(&app);
  super::adapter::set_frame_visible(&target, show)
}
