// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[path = "commands/composition.rs"]
mod composition;
pub use composition::set_recording_preview_composition;
pub use composition::set_recording_preview_cursor_effects;
pub use composition::{
  __cmd__set_recording_preview_composition, __tauri_command_name_set_recording_preview_composition,
};
pub use composition::{
  __cmd__set_recording_preview_cursor_effects,
  __tauri_command_name_set_recording_preview_cursor_effects,
};

#[path = "commands/startup.rs"]
mod startup;
pub use startup::start_recording_preview_player;
pub use startup::{
  __cmd__start_recording_preview_player, __tauri_command_name_start_recording_preview_player,
};

use crate::editor::CameraOverlaySettings;
use tauri::{ipc::Channel, AppHandle, Emitter, Manager};

use super::*;

pub(crate) mod playback;

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct RecordingPreviewSelectionChangeEvent {
  pane_index: Option<u32>,
  session_id: u64,
}
/// A right press on a video layer in the native canvas. The point is in
/// logical px from the top-left of the window's content, the frame the webview
/// reports `clientX`/`clientY` in, so the menu opens under the pointer.
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct RecordingPreviewContextMenuEvent {
  pane_index: u32,
  x: f64,
  y: f64,
}
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct RecordingPreviewSelectionGestureEvent {
  camera_overlay: Option<CameraOverlaySettings>,
  delta_x: f64,
  delta_y: f64,
  edges: u32,
  operation: u32,
  pane_index: u32,
  phase: &'static str,
  recording_output: Option<RecordingOutputSettings>,
  scale: f64,
  session_id: u64,
}

/// Playback teardown kills the ffmpeg audio child, drops the CoreAudio output
/// stream and joins the decode threads, which together take the better part of
/// a second. Never do that on the caller's thread: these commands run on the
/// macOS main thread, and WebKit's layer commits go through it, so the webview
/// visibly freezes for as long as the join lasts.
async fn cancel_off_thread(worker: PreviewPlayerWorker) -> Result<(), String> {
  tauri::async_runtime::spawn_blocking(move || {
    // The position the caller already read from the atomic is at most one
    // frame behind what the join reports, so the returned value is dropped.
    worker.cancel();
  })
  .await
  .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn pause_recording_preview(
  state: tauri::State<'_, RecordingPreviewPlayerState>,
  session_id: u64,
) -> Result<(), String> {
  let (worker, result) = {
    let mut manager = state
      .0
      .lock()
      .map_err(|_| "The recording preview player is unavailable".to_owned())?;
    manager.require_session(session_id)?;
    manager.is_playing = false;
    // Taking the worker signals it - so it stops presenting frames before the
    // still decoder starts - and records the position it displayed, without
    // waiting for its threads. The still decoder is an independent reader, so
    // the paused frame decodes in parallel with that teardown.
    let worker = manager.take_worker();
    let displayed_position_ms = manager.position_ms;
    let duration_ms = manager
      .sources
      .as_ref()
      .map_or(0, |sources| sources.duration_ms);
    manager.position_ms = decodable_position(manager.position_ms, duration_ms);
    if let Some(channel) = &manager.event_channel {
      let _ = channel.send(RecordingPreviewPlayerEvent::Paused {
        position_ms: displayed_position_ms,
      });
    }
    (worker, manager.restart(PlaybackMode::InteractiveStill))
  };
  if let Some(worker) = worker {
    cancel_off_thread(worker).await?;
  }
  result
}

fn decodable_position(position_ms: u64, duration_ms: u64) -> u64 {
  position_ms.min(duration_ms.saturating_sub(1))
}

#[cfg(test)]
mod tests {
  use super::decodable_position;

  #[test]
  fn playback_end_uses_the_final_decodable_offset_for_its_still() {
    assert_eq!(decodable_position(8_000, 8_000), 7_999);
    assert_eq!(decodable_position(4_000, 8_000), 4_000);
    assert_eq!(decodable_position(0, 0), 0);
  }
}

#[tauri::command]
pub async fn seek_recording_preview(
  annotation_clips: Option<Vec<crate::editor::annotations::timing::RecordingAnnotationClip>>,
  state: tauri::State<'_, RecordingPreviewPlayerState>,
  position_ms: u64,
  request_id: u64,
  rough: bool,
  selection_visible: Option<bool>,
  session_id: u64,
) -> Result<(), String> {
  // The request-id guard and the restart stay in one lock scope, so two
  // overlapping seeks still order correctly; only the joins - of separate,
  // already signalled workers - happen afterwards and off this thread.
  let (worker, result) = {
    let mut manager = state
      .0
      .lock()
      .map_err(|_| "The recording preview player is unavailable".to_owned())?;
    manager.require_session(session_id)?;
    if request_id < manager.latest_seek_request {
      return Ok(());
    }
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    if let Some(visible) = selection_visible {
      if let Some(surface) = manager
        .sources
        .as_ref()
        .and_then(|sources| sources.preview_surface.as_ref())
      {
        surface.set_selection_visible(visible);
      }
    }
    if let Some(clips) = annotation_clips {
      crate::editor::annotations::timing::validate_clips(&clips)?;
      if let Some(sources) = &manager.sources {
        *sources
          .annotation_clips
          .write()
          .map_err(|_| "The annotations are unavailable")? = clips;
      }
    }
    manager.latest_seek_request = request_id;
    manager.rough_seek = rough;
    let worker = manager.take_worker();
    let duration_ms = manager
      .sources
      .as_ref()
      .map_or(0, |value| value.duration_ms);
    manager.position_ms = position_ms.min(duration_ms.saturating_sub(1));
    manager.is_playing = false;
    #[cfg(target_os = "macos")]
    manager.publish_annotation_handles();
    (worker, manager.restart(PlaybackMode::InteractiveStill))
  };
  if let Some(worker) = worker {
    cancel_off_thread(worker).await?;
  }
  result
}

#[tauri::command]
pub fn select_recording_preview_audio(
  state: tauri::State<'_, RecordingPreviewPlayerState>,
  enabled_stream_indices: Vec<usize>,
  session_id: u64,
) -> Result<(), String> {
  let mut manager = state
    .0
    .lock()
    .map_err(|_| "The recording preview player is unavailable".to_owned())?;
  manager.require_session(session_id)?;
  manager.audio_indices.clone_from(&enabled_stream_indices);
  if let Some(worker) = &manager.worker {
    worker.select_audio(enabled_stream_indices)?;
  }
  Ok(())
}

#[tauri::command]
pub fn set_recording_preview_audio_volumes(
  state: tauri::State<'_, RecordingPreviewPlayerState>,
  audio_track_volumes: Vec<AudioTrackVolume>,
  session_id: u64,
) -> Result<(), String> {
  let mut manager = state
    .0
    .lock()
    .map_err(|_| "The recording preview player is unavailable".to_owned())?;
  manager.require_session(session_id)?;
  manager.audio_volumes.clone_from(&audio_track_volumes);
  if let Some(worker) = &manager.worker {
    worker.set_audio_volumes(audio_track_volumes)?;
  }
  Ok(())
}

#[tauri::command]
pub async fn stop_recording_preview_player(
  state: tauri::State<'_, RecordingPreviewPlayerState>,
  session_id: u64,
) -> Result<(), String> {
  let worker = {
    let mut manager = state
      .0
      .lock()
      .map_err(|_| "The recording preview player is unavailable".to_owned())?;
    if manager.session_id != Some(session_id) {
      return Ok(());
    }
    // The rest of the teardown - hiding the surface, releasing the sources -
    // has to stay under the lock so a later session cannot start on top of it;
    // only the playback worker's join moves off this thread.
    let worker = manager.take_worker();
    manager.stop();
    worker
  };
  if let Some(worker) = worker {
    cancel_off_thread(worker).await?;
  }
  Ok(())
}

pub fn stop_all(app: &AppHandle) {
  if let Some(state) = app.try_state::<RecordingPreviewPlayerState>() {
    if let Ok(mut manager) = state.0.lock() {
      manager.stop();
    }
  }
}
