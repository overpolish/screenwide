// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(test)]
#[path = "camera_preview/tests.rs"]
mod tests;

#[path = "camera_preview/camera_worker.rs"]
mod camera_worker;
#[path = "camera_preview/delivery.rs"]
mod delivery;
use camera_worker::build_camera_preview;

use delivery::PreviewDelivery;
#[cfg(test)]
use delivery::{frame_payload, preview_dimensions};

use std::{
  sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc, Mutex,
  },
  time::Duration,
  time::Instant,
};

use nokhwa::{
  pixel_format::RgbAFormat,
  query,
  utils::{ApiBackend, FrameFormat, RequestedFormat, RequestedFormatType},
  Buffer, CallbackCamera,
};

#[cfg(target_os = "macos")]
use nokhwa::utils::{CameraFormat, Resolution};
use tauri::{
  ipc::{Channel, InvokeResponseBody},
  AppHandle, Manager,
};

use crate::recording_inputs::camera_id;

#[cfg(target_os = "macos")]
use crate::camera_frame_rate;

#[cfg(not(target_os = "macos"))]
use crate::camera_format::resolve_exact_camera_format;

/// The session most recently taken down, kept so the next start can tell
/// whether it follows hot on the heels of one on the same device.
#[derive(Clone)]
struct EndedSession {
  device_id: String,
  fps: u32,
  ended_at: Instant,
}

#[derive(Default)]
struct CameraPreviewManager {
  worker: Option<CameraPreviewWorker>,
  generation: u64,
  last_ended: Option<EndedSession>,
}

impl CameraPreviewManager {
  /// Claims the next generation and hands back the worker it supersedes. The
  /// caller tears that worker down off the state lock so a slow camera close
  /// never blocks the thread that is holding it.
  fn begin_start(&mut self) -> (u64, Option<CameraPreviewWorker>, Option<EndedSession>) {
    self.generation = self.generation.wrapping_add(1);
    let previous = self.worker.take();
    if let Some(previous) = &previous {
      self.note_ended(previous);
    }
    (self.generation, previous, self.last_ended.clone())
  }

  fn note_ended(&mut self, worker: &CameraPreviewWorker) {
    self.last_ended = Some(EndedSession {
      device_id: worker.device_id.clone(),
      fps: worker.fps,
      ended_at: Instant::now(),
    });
  }

  /// Stores the worker when it still matches the current generation, otherwise
  /// returns it so the caller can cancel it away from the lock.
  fn finish_start(
    &mut self,
    generation: u64,
    worker: CameraPreviewWorker,
  ) -> Option<CameraPreviewWorker> {
    if self.generation == generation {
      self.worker = Some(worker);
      None
    } else {
      Some(worker)
    }
  }

  fn take_worker(&mut self) -> Option<CameraPreviewWorker> {
    self.generation = self.generation.wrapping_add(1);
    let worker = self.worker.take();
    if let Some(worker) = &worker {
      self.note_ended(worker);
    }
    worker
  }

  fn cancel(&mut self) {
    if let Some(worker) = self.take_worker() {
      worker.cancel();
    }
  }
}

struct CameraPreviewWorker {
  cancelled: Arc<AtomicBool>,
  delivery: Option<PreviewDelivery>,
  device_id: String,
  fps: u32,
  thread: Option<std::thread::JoinHandle<()>>,
}

impl CameraPreviewWorker {
  fn cancel(mut self) {
    self.cancelled.store(true, Ordering::Release);
    if let Some(thread) = self.thread.take() {
      let _ = thread.join();
    }
    if let Some(delivery) = self.delivery.take() {
      delivery.stop();
    }
  }
}

#[derive(Default)]
pub struct CameraPreviewState(Mutex<CameraPreviewManager>);

const PREVIEW_MAX_WIDTH: u32 = 384;
const PREVIEW_MAX_HEIGHT: u32 = 240;
const PREVIEW_INTERVAL: Duration = Duration::from_millis(16);

const DELIVERY_POLL_INTERVAL: Duration = Duration::from_millis(100);
/// How long a Continuity Camera takes to close its phone-side stream once the
/// last session on it ends; only then does it accept a higher frame rate.
const CONTINUITY_CAMERA_COLD_START: Duration = Duration::from_millis(3500);

/// The least time a device is given between one session ending and the next
/// opening on it. Opened sooner, as when the preview's resolution is switched
/// and the new session follows the old one within milliseconds, the device
/// accepts the session but has not let go of the old one: it delivers either
/// nothing or black frames until it is closed and opened again after a pause.
/// Switching the input off and on by hand always leaves at least this long.
const DEVICE_REOPEN_GAP: Duration = Duration::from_millis(1000);

/// Whether `device_id` is a camera that keeps its last frame rate across
/// sessions (Continuity Camera).
#[cfg(target_os = "macos")]
fn camera_frame_rate_is_sticky(device_id: &str) -> bool {
  camera_frame_rate::resolve_device(device_id, "").is_ok_and(|device| device.is_continuity_camera())
}

#[cfg(not(target_os = "macos"))]
fn camera_frame_rate_is_sticky(_device_id: &str) -> bool {
  false
}

#[tauri::command]
pub async fn start_camera_preview(
  state: tauri::State<'_, CameraPreviewState>,
  device_id: String,
  width: u32,
  height: u32,
  fps: u32,
  pal: bool,
  channel: Channel,
) -> Result<(), String> {
  let (generation, previous, last_ended) = state
    .0
    .lock()
    .map_err(|_| "Camera preview state is unavailable".to_owned())?
    .begin_start();
  let worker = tauri::async_runtime::spawn_blocking(move || {
    // The previous worker has to release the device before the new camera is
    // opened, otherwise the same device can still be busy.
    if let Some(previous) = previous {
      previous.cancel();
    }
    // Continuity Camera keeps the phone-side pipeline warm for a few seconds
    // after a session ends, and a session started on it in that window
    // inherits the old frame rate: it follows a lower rate live, but never a
    // higher one. Letting the phone go fully idle first is what a
    // close-and-reopen of the options does, and the only thing that works.
    // The previous session is usually stopped by the front end before this
    // start arrives, hence the manager's memory rather than `previous`.
    let cold_start_wait = last_ended
      .as_ref()
      .filter(|ended| ended.device_id == device_id && fps > ended.fps)
      .and_then(|ended| CONTINUITY_CAMERA_COLD_START.checked_sub(ended.ended_at.elapsed()))
      .filter(|_| camera_frame_rate_is_sticky(&device_id));
    let reopen_wait = last_ended
      .as_ref()
      .filter(|ended| ended.device_id == device_id)
      .and_then(|ended| DEVICE_REOPEN_GAP.checked_sub(ended.ended_at.elapsed()));
    let wait = match (cold_start_wait, reopen_wait) {
      (Some(cold), Some(reopen)) => Some(cold.max(reopen)),
      (cold, reopen) => cold.or(reopen),
    };
    if let Some(wait) = wait {
      std::thread::sleep(wait);
    }
    build_camera_preview(&device_id, width, height, fps, pal, channel)
  })
  .await
  .map_err(|error| error.to_string())??;
  let stale = state
    .0
    .lock()
    .map_err(|_| "Camera preview state is unavailable".to_owned())?
    .finish_start(generation, worker);
  if let Some(stale) = stale {
    cancel_off_thread(stale).await?;
  }
  Ok(())
}

/// Camera teardown joins worker threads that can block for a while, so it must
/// never run on the caller's thread: `stop_camera_preview` is invoked on the
/// macOS main thread and would freeze the UI.
async fn cancel_off_thread(worker: CameraPreviewWorker) -> Result<(), String> {
  tauri::async_runtime::spawn_blocking(move || worker.cancel())
    .await
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn stop_camera_preview(
  state: tauri::State<'_, CameraPreviewState>,
) -> Result<(), String> {
  let worker = state
    .0
    .lock()
    .map_err(|_| "Camera preview state is unavailable".to_owned())?
    .take_worker();
  if let Some(worker) = worker {
    cancel_off_thread(worker).await?;
  }
  Ok(())
}

pub fn stop_all(app: &AppHandle) {
  if let Some(state) = app.try_state::<CameraPreviewState>() {
    if let Ok(mut manager) = state.0.lock() {
      manager.cancel();
    }
  }
}
