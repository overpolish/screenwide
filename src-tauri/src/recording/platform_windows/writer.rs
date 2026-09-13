// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[path = "writer/frames.rs"]
mod frames;
#[path = "writer/media_types.rs"]
mod media_types;
#[path = "writer/sink.rs"]
mod sink;
#[path = "writer/timeline.rs"]
mod timeline;
use frames::crop_frame;
pub(super) use frames::snapshot_frame;
use media_types::{attributes, encoder_config, video_type};

use std::path::PathBuf;
use std::sync::{mpsc, Arc, OnceLock};
use std::time::{Duration, Instant};

use windows::core::{Interface, PCWSTR};
use windows::Win32::Foundation::PROPERTYKEY;
use windows::Win32::Graphics::Direct3D11::{
  ID3D11Device, ID3D11Resource, ID3D11Texture2D, D3D11_BIND_RENDER_TARGET,
  D3D11_BIND_SHADER_RESOURCE, D3D11_BOX, D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT,
};
use windows::Win32::Media::MediaFoundation::*;
use windows::Win32::System::Com::StructuredStorage::PROPVARIANT;
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};
use windows::Win32::System::Variant::VT_UI4;
use windows::Win32::UI::Shell::PropertiesSystem::{IPropertyStore, PSCreateMemoryPropertyStore};

use crate::capture_geometry::CaptureRect;
use crate::recording::encoding::{bitrate_bps, FailureReport, FinalizeInfo, Timeline};
use crate::recording::PrimaryRecordingKind;

const NANOS_PER_100NS: i64 = 100;

fn frame_cadence(fps: u32) -> Duration {
  Duration::from_nanos(1_000_000_000_u64 / u64::from(fps.max(1)))
}

fn win<T>(result: windows::core::Result<T>) -> Result<T, String> {
  result.map_err(|error| error.to_string())
}

#[derive(Clone)]
pub(super) struct Frame {
  pub(super) source_100ns: i64,
  pub(super) texture: ID3D11Texture2D,
  pub(super) wall: Instant,
}

pub(super) enum Command {
  Frame(Frame),
  Pause(Instant),
  Resume(Instant),
  Stop {
    at: Instant,
    reply: mpsc::Sender<Result<FinalizeInfo, String>>,
  },
  Cancel,
}

pub(super) struct WriterConfig {
  pub(super) device: ID3D11Device,
  pub(super) fps: u32,
  pub(super) height: u32,
  pub(super) on_failure: FailureReport,
  pub(super) path: PathBuf,
  pub(super) primary_kind: PrimaryRecordingKind,
  pub(super) source_crop: Option<CaptureRect>,
  pub(super) establish_timeline_origin: bool,
  pub(super) stopped_at: Arc<OnceLock<Instant>>,
  pub(super) timeline_origin: Arc<OnceLock<Instant>>,
  pub(super) wall_timestamped_frames: bool,
  pub(super) width: u32,
}

struct MediaFoundation;

impl MediaFoundation {
  fn start() -> Result<Self, String> {
    unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }
      .ok()
      .map_err(|error| error.to_string())?;
    if let Err(error) = unsafe { MFStartup(MF_VERSION, MFSTARTUP_FULL) } {
      unsafe { CoUninitialize() };
      return Err(error.to_string());
    }
    Ok(Self)
  }
}

impl Drop for MediaFoundation {
  fn drop(&mut self) {
    let _ = unsafe { MFShutdown() };
    unsafe { CoUninitialize() };
  }
}

struct Sink {
  _byte_stream: IMFByteStream,
  _device_manager: IMFDXGIDeviceManager,
  media_sink: IMFMediaSink,
  sink: Option<IMFSinkWriter>,
  stream: u32,
}

struct Writer {
  base: Instant,
  config: WriterConfig,
  failed: Option<String>,
  frame_duration_100ns: i64,
  last_appended_ns: Option<i64>,
  sink: Sink,
  tail: Option<Frame>,
  timeline: Timeline,
}

fn after_stop(stopped_at: &OnceLock<Instant>, frame_at: Instant) -> bool {
  stopped_at
    .get()
    .is_some_and(|stopped_at| frame_at > *stopped_at)
}

pub(super) fn run(
  config: WriterConfig,
  commands: mpsc::Receiver<Command>,
  initialized: mpsc::Sender<Result<(), String>>,
  first_frame: mpsc::Sender<Result<(), String>>,
) {
  let _media_foundation = match MediaFoundation::start() {
    Ok(runtime) => runtime,
    Err(error) => {
      let _ = initialized.send(Err(error.clone()));
      let _ = first_frame.send(Err(error));
      return;
    }
  };
  let mut writer = match Writer::new(config) {
    Ok(writer) => writer,
    Err(error) => {
      let _ = initialized.send(Err(error.clone()));
      let _ = first_frame.send(Err(error));
      return;
    }
  };
  let _ = initialized.send(Ok(()));
  let mut announced = false;
  let cadence = frame_cadence(writer.config.fps);
  let mut next_tick: Option<Instant> = None;
  let mut pending = None;
  loop {
    // A continuously changing window can keep the frame channel readable at
    // all times. Honour an elapsed presentation deadline before reading more
    // capture work so incoming WGC frames cannot starve the fixed-rate clock.
    if let Some(deadline) = next_tick {
      if Instant::now() >= deadline {
        writer.tick(deadline);
        next_tick = Some(deadline + cadence);
        continue;
      }
    }
    let command = match pending.take() {
      Some(command) => Some(command),
      None => match next_tick {
        Some(deadline) => {
          match commands.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
            Ok(command) => Some(command),
            Err(mpsc::RecvTimeoutError::Timeout) => {
              writer.tick(deadline);
              next_tick = Some(deadline + cadence);
              None
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => return,
          }
        }
        None => match commands.recv() {
          Ok(command) => Some(command),
          Err(_) => return,
        },
      },
    };
    let Some(command) = command else { continue };
    match command {
      Command::Frame(mut frame) => {
        // Window and cropped-region capture only need the newest texture
        // before the next presentation tick. Discarding stale queued surfaces
        // prevents startup or crop/encoder pressure from becoming a permanent
        // multi-frame cursor delay. Full-screen and camera capture retain
        // every frame and keep their existing source-timestamp path.
        if writer.config.wall_timestamped_frames {
          loop {
            match commands.try_recv() {
              Ok(Command::Frame(newer)) => frame = newer,
              Ok(command) => {
                pending = Some(command);
                break;
              }
              Err(mpsc::TryRecvError::Empty) => break,
              Err(mpsc::TryRecvError::Disconnected) => return,
            }
          }
        }
        if writer.frame(frame) && !announced {
          announced = true;
          let _ = first_frame.send(Ok(()));
          if writer.config.wall_timestamped_frames {
            next_tick = Some(Instant::now() + cadence);
          }
        } else if !announced {
          if let Some(error) = writer.failed.clone() {
            let _ = first_frame.send(Err(error));
            return;
          }
        }
      }
      Command::Pause(at) => {
        let elapsed = writer.elapsed_ns(at);
        writer.timeline.pause(elapsed);
        if writer.config.wall_timestamped_frames {
          next_tick = None;
        }
      }
      Command::Resume(at) => {
        let elapsed = writer.elapsed_ns(at);
        writer.timeline.resume(elapsed);
        if writer.config.wall_timestamped_frames && writer.timeline.has_started() {
          next_tick = Some(at + cadence);
        }
      }
      Command::Stop { at, reply } => {
        let _ = reply.send(writer.finish(at));
        return;
      }
      Command::Cancel => return,
    }
  }
}

#[cfg(test)]
mod tests {
  use std::time::Duration;

  use super::*;

  #[test]
  fn rejects_frames_captured_after_the_user_pressed_stop() {
    let base = Instant::now();
    let stopped_at = OnceLock::new();
    assert!(!after_stop(&stopped_at, base + Duration::from_secs(10)));
    stopped_at.set(base + Duration::from_secs(1)).unwrap();
    assert!(!after_stop(&stopped_at, base + Duration::from_secs(1)));
    assert!(after_stop(
      &stopped_at,
      base + Duration::from_secs(1) + Duration::from_nanos(1)
    ));
  }

  #[test]
  fn repeat_cadence_matches_selected_frame_rate() {
    assert_eq!(frame_cadence(30), Duration::from_nanos(33_333_333));
    assert_eq!(frame_cadence(60), Duration::from_nanos(16_666_666));
  }
}
