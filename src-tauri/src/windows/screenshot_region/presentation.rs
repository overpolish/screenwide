// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::AppHandle;
#[cfg(target_os = "macos")]
use tauri::Manager;

/// Leaves the recording Region shade/cutout visible while temporarily hiding
/// its interactive OSC during a still or scrolling capture.
#[cfg(target_os = "macos")]
#[tauri::command]
pub fn set_region_selector_osc_frame_visible(
  app: AppHandle,
  visible: bool,
) -> Result<bool, String> {
  let target = app
    .get_webview_window(super::super::WindowLabel::RegionSelector.as_str())
    .ok_or_else(|| "Region selector not found".to_owned())?;
  let show = visible && crate::recording::is_idle(&app);
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  app
    .run_on_main_thread(move || {
      let result = target
        .ns_view()
        .map(|view| super::native_osc_macos::set_show_frame(view.cast(), show))
        .map_err(|error| error.to_string());
      let _ = sender.send(result);
    })
    .map_err(|error| error.to_string())?;
  receiver.recv().map_err(|error| error.to_string())?
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
pub fn set_region_selector_osc_frame_visible(
  _app: AppHandle,
  _visible: bool,
) -> Result<bool, String> {
  Ok(false)
}
