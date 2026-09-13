// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows camera capture into the native Media Foundation video writer.
//!
//! Media Foundation exposes ordinary webcams to Nokhwa as native YUYV/MJPEG
//! buffers. Decoding those device-owned bytes is the one unavoidable CPU
//! boundary; the resulting BGRA frame is uploaded directly to D3D11 and the
//! existing hardware H.264 writer owns everything downstream.

mod confidence;

#[cfg(test)]
#[path = "camera/tests.rs"]
mod tests;

#[path = "camera/frames.rs"]
mod frames;
use frames::{bgra_pixels, camera_frame_clock, report_once, texture};

use std::sync::{
  atomic::{AtomicBool, Ordering},
  mpsc, Arc, OnceLock,
};
use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};

use nokhwa::{
  pixel_format::RgbAFormat,
  query,
  utils::{ApiBackend, RequestedFormat, RequestedFormatType},
  CallbackCamera,
};
use rayon::prelude::*;
use windows::Win32::Graphics::{
  Direct3D11::{
    ID3D11Device, ID3D11Texture2D, D3D11_BIND_RENDER_TARGET, D3D11_BIND_SHADER_RESOURCE,
    D3D11_SUBRESOURCE_DATA, D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT,
  },
  Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
};

use super::writer::{Command, Frame};
use crate::camera_format::resolve_exact_camera_format;
use crate::recording::{encoding::FailureReport, monitor::RecordingMonitor, CameraCaptureMode};
use crate::recording_inputs::camera_id;

const START_TIMEOUT: Duration = Duration::from_secs(8);
const WARMUP_DURATION: Duration = Duration::from_millis(500);
const WARMUP_MIN_FRAMES: u64 = 4;

pub(super) struct CameraSpec {
  device_id: String,
  index: nokhwa::utils::CameraIndex,
  pub(super) flipped: bool,
  pub(super) fps: u32,
  pub(super) height: u32,
  pal: bool,
  pub(super) width: u32,
}

impl CameraSpec {
  pub(super) fn resolve(mode: CameraCaptureMode) -> Result<Self, String> {
    let info = query(ApiBackend::Auto)
      .map_err(|error| error.to_string())?
      .into_iter()
      .find(|info| camera_id(info) == mode.device_id)
      .ok_or_else(|| "The selected camera is no longer available".to_owned())?;
    let width = mode.width & !1;
    let height = mode.height & !1;
    if width < 2 || height < 2 {
      return Err("The selected camera mode has no recordable area".to_owned());
    }
    Ok(Self {
      device_id: mode.device_id,
      index: info.index().clone(),
      flipped: mode.flipped,
      fps: mode.fps.max(1),
      height,
      pal: mode.pal,
      width,
    })
  }
}

pub(super) struct CameraStream {
  cancelled: Arc<AtomicBool>,
  confidence: Option<confidence::ConfidenceWorker>,
  worker: Option<std::thread::JoinHandle<()>>,
}

impl CameraStream {
  pub(super) fn stop(mut self) {
    self.cancelled.store(true, Ordering::Release);
    if let Some(worker) = self.worker.take() {
      let _ = worker.join();
    }
    if let Some(confidence) = self.confidence.take() {
      confidence.stop();
    }
  }
}

impl Drop for CameraStream {
  fn drop(&mut self) {
    self.cancelled.store(true, Ordering::Release);
    if let Some(worker) = self.worker.take() {
      let _ = worker.join();
    }
    if let Some(confidence) = self.confidence.take() {
      confidence.stop();
    }
  }
}

struct CallbackState {
  first_frame_at: Option<Instant>,
  frame_count: u64,
  warmup_announced: bool,
}

fn warmup_complete(frame_count: u64, elapsed: Duration) -> bool {
  frame_count >= WARMUP_MIN_FRAMES && elapsed >= WARMUP_DURATION
}

pub(super) fn start(
  spec: CameraSpec,
  device: ID3D11Device,
  commands: mpsc::SyncSender<Command>,
  timeline_origin: Arc<OnceLock<Instant>>,
  monitor: Arc<RecordingMonitor>,
  on_failure: FailureReport,
) -> Result<CameraStream, String> {
  let format = resolve_exact_camera_format(&spec.index, spec.width, spec.height, spec.fps)?;
  // Anti-flicker lives in the camera on Windows, not in the cadence; a camera
  // without the control (virtual cameras) still records, so this only reports.
  if let Err(error) =
    crate::camera_power_line::apply_power_line_frequency(&spec.device_id, spec.pal)
  {
    eprintln!("The camera's power line frequency was not set: {error}");
  }
  let confidence = confidence::ConfidenceWorker::spawn(Arc::clone(&monitor))?;
  let confidence_frames = confidence.sender();
  let cancelled = Arc::new(AtomicBool::new(false));
  let owner_cancelled = Arc::clone(&cancelled);
  let callback_cancelled = Arc::clone(&cancelled);
  let failure_reported = Arc::new(AtomicBool::new(false));
  let callback_failure_reported = Arc::clone(&failure_reported);
  let (started_tx, started) = mpsc::channel();
  let worker = std::thread::Builder::new()
    .name("screenwide-camera-capture-windows".to_owned())
    .spawn(move || {
      let requested = RequestedFormat::new::<RgbAFormat>(RequestedFormatType::Exact(format));
      let capture_started = Instant::now();
      let mut state = CallbackState {
        first_frame_at: None,
        frame_count: 0,
        warmup_announced: false,
      };
      let mut camera = match CallbackCamera::new(spec.index, requested, move |frame| {
        if callback_cancelled.load(Ordering::Acquire) {
          return;
        }
        let wall = Instant::now();
        let (frame_wall, source_100ns) = camera_frame_clock(
          frame.capture_timestamp(),
          wall,
          SystemTime::now().duration_since(UNIX_EPOCH).ok(),
          capture_started,
        );
        let first_frame_at = *state.first_frame_at.get_or_insert(wall);
        state.frame_count = state.frame_count.saturating_add(1);
        if !warmup_complete(state.frame_count, wall.duration_since(first_frame_at)) {
          return;
        }
        if !state.warmup_announced {
          state.warmup_announced = true;
          // The camera's warm-up boundary is the shared zero for every track.
          // Screen and audio samples captured before it are discarded, keeping
          // the finished streams aligned without delaying backend startup.
          let _ = timeline_origin.set(frame_wall);
        }
        let resolution = frame.resolution();
        if (resolution.width(), resolution.height()) != (spec.width, spec.height) {
          report_once(
            &callback_failure_reported,
            &on_failure,
            format!(
              "The camera delivered {} x {} instead of the selected {} x {} format",
              resolution.width(),
              resolution.height(),
              spec.width,
              spec.height,
            ),
          );
          callback_cancelled.store(true, Ordering::Release);
          return;
        }
        let result = frame
          .decode_image::<RgbAFormat>()
          .map_err(|error| error.to_string())
          .map(|image| image.into_raw())
          .and_then(|rgba| {
            let rgba = Arc::new(rgba);
            if monitor.is_subscribed() {
              let _ = confidence_frames.try_send(confidence::CameraFrame {
                flipped: spec.flipped,
                height: spec.height,
                rgba: Arc::clone(&rgba),
                width: spec.width,
              });
            }
            let bgra = bgra_pixels(&rgba, spec.width, spec.height, spec.flipped);
            texture(&device, spec.width, spec.height, &bgra)
          });
        match result {
          Ok(texture) => {
            match commands.try_send(Command::Frame(Frame {
              source_100ns,
              texture,
              wall: frame_wall,
            })) {
              Ok(()) | Err(mpsc::TrySendError::Full(_)) => {}
              Err(mpsc::TrySendError::Disconnected(_)) => {
                callback_cancelled.store(true, Ordering::Release);
              }
            }
          }
          Err(error) => report_once(&callback_failure_reported, &on_failure, error),
        }
      }) {
        Ok(camera) => camera,
        Err(error) => {
          let _ = started_tx.send(Err(error.to_string()));
          return;
        }
      };
      if let Err(error) = camera.open_stream() {
        let _ = started_tx.send(Err(error.to_string()));
        return;
      }
      if started_tx.send(Ok(())).is_err() {
        return;
      }
      while !owner_cancelled.load(Ordering::Acquire) {
        std::thread::sleep(Duration::from_millis(5));
      }
      drop(camera);
    })
    .map_err(|error| error.to_string())?;

  match started.recv_timeout(START_TIMEOUT) {
    Ok(Ok(())) => Ok(CameraStream {
      cancelled,
      confidence: Some(confidence),
      worker: Some(worker),
    }),
    Ok(Err(error)) => {
      cancelled.store(true, Ordering::Release);
      let _ = worker.join();
      confidence.stop();
      Err(error)
    }
    Err(_) => {
      cancelled.store(true, Ordering::Release);
      let _ = worker.join();
      confidence.stop();
      Err("The camera did not start in time".to_owned())
    }
  }
}
