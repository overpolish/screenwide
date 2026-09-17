// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Unwinds a start that could not be completed, from wherever it failed.
pub(super) fn abandon_start(app: &AppHandle, error: &str) {
  emit_error(app, "start", error);
  state(app).cancel();
  discard_capture(take_handles(app));
  restore_windows(app);
  let _ = transition(app, RecordingStatus::Idle, None);
  crate::editor::release_recording_workspace(app);
  show_recording_ui(app);
}

pub fn start(app: &AppHandle, options: StartRecordingOptions) -> Result<(), String> {
  crate::capture_overlays::dismiss_all(app);
  validate_options(&options)?;
  crate::editor::reserve_recording_workspace(app)?;
  // A second start while `Starting` is rejected here, not merely by a
  // disabled button.
  if let Err(error) = transition(app, RecordingStatus::Starting, Some(options.mode)) {
    crate::editor::release_recording_workspace(app);
    return Err(error);
  }
  let generation = state(app).begin_start();
  let countdown_seconds = crate::settings::current(app).recording_countdown_seconds;

  if let Err(error) = prepare_windows(app, &options) {
    abandon_start(app, &error);
    return Err(error);
  }
  // The capture only opens after the countdown, but the dock sizes itself
  // from the monitor's source flags - announcing the planned inputs now lets
  // it show the confidence layout (and its width) through the countdown.
  state(app).monitor.configure(
    options.system_audio,
    options.microphone_id.is_some(),
    options.camera_id.is_some(),
  );

  let app = app.clone();
  // Opening a capture talks to the window server and waits on it. `tokio` is
  // macOS-only in this crate, so this is a blocking task the way finalize is -
  // and either way it must not run on the thread that draws.
  tauri::async_runtime::spawn_blocking(move || {
    for seconds in (1..=countdown_seconds).rev() {
      if !state(&app).is_current(generation) {
        return;
      }
      set_countdown(&app, seconds);
      std::thread::sleep(std::time::Duration::from_secs(1));
    }
    set_countdown(&app, 0);
    if !state(&app).is_current(generation) {
      return;
    }
    // Capture exclusion is a compositor state change, and a countdown is
    // normally long enough for DWM to present it. Without one the first
    // frames could still carry a window that is meant to be absent - most
    // visibly a live annotation, which the recording also receives as clips.
    #[cfg(target_os = "windows")]
    if countdown_seconds == 0 {
      std::thread::sleep(std::time::Duration::from_millis(75));
    }

    let (handles, first_frame) = match begin_capture(&app, &options) {
      Ok(started) => started,
      Err(error) => {
        if state(&app).is_current(generation) {
          abandon_start(&app, &error);
        }
        return;
      }
    };
    // Cancelling while the capture was opening: the handles were never
    // stored, so this is the only place that can still tear them down.
    if !state(&app).is_current(generation) {
      return discard_capture(Some(handles));
    }
    store_handles(&app, handles);

    // Nothing is recording until a frame has actually been written. Moving to
    // `Recording` any earlier would start a clock the file cannot honour.
    let confirmed = first_frame
      .recv_timeout(FIRST_FRAME_TIMEOUT)
      .unwrap_or_else(|_| Err("The recording produced no frames".to_owned()));
    if !state(&app).is_current(generation) {
      // Cancelling usually takes the handles itself, but it can land in the
      // instant between the check above and the store, in which case they are
      // still here and nothing else will ever come back for them.
      return discard_capture(take_handles(&app));
    }

    match confirmed {
      Ok(()) => {
        if let Err(error) = transition(&app, RecordingStatus::Recording, None) {
          emit_error(&app, "start", &error);
          return;
        }
        if let Err(error) = windows::show_recording_dock(&app) {
          emit_error(&app, "start", &error.to_string());
        }
      }
      Err(error) => abandon_start(&app, &error),
    }
  });

  Ok(())
}
