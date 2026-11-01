// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

mod estimate_command;
mod export_inputs;
mod export_preview_command;
mod snapshot_command;
pub use estimate_command::estimate_recording_export;
pub use export_preview_command::{preview_recording_export_frame, preview_screenshot_export};

pub use snapshot_command::get_editor_snapshot;

use super::*;

pub use estimate_command::{
  __cmd__estimate_recording_export, __tauri_command_name_estimate_recording_export,
};
pub use export_preview_command::{
  __cmd__preview_recording_export_frame, __cmd__preview_screenshot_export,
  __tauri_command_name_preview_recording_export_frame,
  __tauri_command_name_preview_screenshot_export,
};

pub use snapshot_command::{__cmd__get_editor_snapshot, __tauri_command_name_get_editor_snapshot};

/// Stops the size estimate and the compared frames still being made. Only
/// the Export island shows them, so once it is put away their GPU work
/// would only hold up the editor's own preview.
pub(crate) fn cancel_export_work(state: &EditorState) {
  use std::sync::atomic::Ordering;
  state
    .running_estimate
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .store(true, Ordering::Release);
  for preview in state
    .running_export_preview
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .iter()
  {
    preview.store(true, Ordering::Release);
  }
}

/// The Export island put away: the estimate and the compared frames it was
/// waiting for are let go.
#[tauri::command]
pub fn stop_export_estimate(app: AppHandle) {
  cancel_export_work(&app.state::<EditorState>());
}
