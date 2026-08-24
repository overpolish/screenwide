// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tauri command adapters for the recording lifecycle and monitor stream.

use tauri::{AppHandle, State};

use super::{
  cancel, pause, resume, start, stop, RecordingSnapshot, RecordingState, StartRecordingOptions,
};

#[tauri::command]
pub fn start_recording_monitor(
  state: State<'_, RecordingState>,
  subscription_id: u64,
  channel: tauri::ipc::Channel,
) {
  state.monitor.subscribe(subscription_id, channel);
}

#[tauri::command]
pub fn stop_recording_monitor(state: State<'_, RecordingState>, subscription_id: u64) {
  state.monitor.unsubscribe(subscription_id);
}

#[tauri::command]
pub fn get_recording_snapshot(state: State<'_, RecordingState>) -> RecordingSnapshot {
  *state
    .snapshot
    .read()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[tauri::command]
pub fn start_recording(app: AppHandle, options: StartRecordingOptions) -> Result<(), String> {
  start(&app, options)
}

#[tauri::command]
pub fn pause_recording(app: AppHandle) -> Result<(), String> {
  pause(&app)
}

#[tauri::command]
pub fn resume_recording(app: AppHandle) -> Result<(), String> {
  resume(&app)
}

#[tauri::command]
pub fn stop_recording(app: AppHandle) -> Result<(), String> {
  stop(&app)
}

#[tauri::command]
pub fn cancel_recording(app: AppHandle) -> Result<(), String> {
  cancel(&app)
}
