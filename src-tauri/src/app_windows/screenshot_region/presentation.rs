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
  let target = app
    .get_webview_window(super::super::WindowLabel::RegionSelector.as_str())
    .ok_or_else(|| "Region selector not found".to_owned())?;
  let show = visible && crate::recording::is_idle(&app);
  super::adapter::set_frame_visible(&target, show)
}
