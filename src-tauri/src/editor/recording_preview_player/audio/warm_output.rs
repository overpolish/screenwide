// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One audio output kept open while the editor is, sounding silence between
//! plays. Bluetooth headphones take a third of a second or more to start an
//! output, which a play opening its own would wait for every time. Each play
//! hands the output what to sound (`Feed`), and takes it back as it ends. It
//! is opened again where the default output device changed or the output
//! failed.

use std::sync::mpsc;

use super::output::build_output;
use super::*;

/// What the output sounds for one play: the samples FFmpeg decoded, the
/// clock that play paces against, and how its channels are mixed.
pub(super) struct Feed {
  pub clock: Arc<AudioClock>,
  pub mix: Mix,
  pub queue: Arc<Mutex<VecDeque<f32>>>,
}

/// Where the output finds what it sounds; none sounds silence.
pub(super) type Slot = Arc<Mutex<Option<Feed>>>;

struct Open {
  config: StreamConfig,
  /// The device's name, to notice the default device changing.
  device: Option<String>,
  failed: Arc<AtomicBool>,
  slot: Slot,
  /// Dropped, it lets the output's thread go, which closes the output.
  _close: mpsc::Sender<()>,
}

static OUTPUT: Mutex<Option<Open>> = Mutex::new(None);

fn device_name(device: &cpal::Device) -> Option<String> {
  device
    .description()
    .ok()
    .map(|description| description.name().to_owned())
}

/// The open output's format and slot, opening it where it is not open, the
/// default output device is another, or it failed.
pub(super) fn output() -> Result<(StreamConfig, Slot), String> {
  let device = cpal::default_host()
    .default_output_device()
    .ok_or_else(|| "No audio output device is available".to_owned())?;
  let name = device_name(&device);
  let mut output = OUTPUT
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  if let Some(open) = output
    .as_ref()
    .filter(|open| open.device == name && !open.failed.load(Ordering::Acquire))
  {
    return Ok((open.config, Arc::clone(&open.slot)));
  }
  *output = None;
  let open = open(device, name)?;
  let found = (open.config, Arc::clone(&open.slot));
  *output = Some(open);
  Ok(found)
}

/// Opens the output in the background, so the first play finds it open.
pub(in crate::editor::recording_preview_player) fn warm() {
  let _ = std::thread::Builder::new()
    .name("recording-preview-audio-warm".to_owned())
    .spawn(|| {
      if let Err(error) = output() {
        eprintln!("The preview's audio output could not be opened: {error}");
      }
    });
}

/// Closes the output, as the editor does.
pub(in crate::editor::recording_preview_player) fn close() {
  *OUTPUT
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
}

/// Opens `device` on a thread of its own, which holds the stream until the
/// output is closed: a stream may not move between threads everywhere.
fn open(device: cpal::Device, name: Option<String>) -> Result<Open, String> {
  let supported = device
    .default_output_config()
    .map_err(|error| error.to_string())?;
  let config: StreamConfig = supported.config();
  let slot: Slot = Arc::new(Mutex::new(None));
  let failed = Arc::new(AtomicBool::new(false));
  let (close, closed) = mpsc::channel::<()>();
  let (opened, result) = mpsc::channel::<Result<(), String>>();
  let thread_slot = Arc::clone(&slot);
  let thread_failed = Arc::clone(&failed);
  std::thread::Builder::new()
    .name("recording-preview-audio-output".to_owned())
    .spawn(move || {
      let built = match supported.sample_format() {
        SampleFormat::F32 => build_output::<f32>(&device, &config, thread_slot, thread_failed),
        SampleFormat::F64 => build_output::<f64>(&device, &config, thread_slot, thread_failed),
        SampleFormat::I8 => build_output::<i8>(&device, &config, thread_slot, thread_failed),
        SampleFormat::I16 => build_output::<i16>(&device, &config, thread_slot, thread_failed),
        SampleFormat::I24 => {
          build_output::<cpal::I24>(&device, &config, thread_slot, thread_failed)
        }
        SampleFormat::I32 => build_output::<i32>(&device, &config, thread_slot, thread_failed),
        SampleFormat::I64 => build_output::<i64>(&device, &config, thread_slot, thread_failed),
        SampleFormat::U8 => build_output::<u8>(&device, &config, thread_slot, thread_failed),
        SampleFormat::U16 => build_output::<u16>(&device, &config, thread_slot, thread_failed),
        SampleFormat::U24 => {
          build_output::<cpal::U24>(&device, &config, thread_slot, thread_failed)
        }
        SampleFormat::U32 => build_output::<u32>(&device, &config, thread_slot, thread_failed),
        SampleFormat::U64 => build_output::<u64>(&device, &config, thread_slot, thread_failed),
        format => Err(format!("Unsupported audio output format: {format}")),
      };
      let stream = match built.and_then(|stream| {
        stream
          .play()
          .map(|()| stream)
          .map_err(|error| error.to_string())
      }) {
        Ok(stream) => stream,
        Err(error) => {
          let _ = opened.send(Err(error));
          return;
        }
      };
      let _ = opened.send(Ok(()));
      // Every sender gone, the output is closed.
      let _ = closed.recv();
      drop(stream);
    })
    .map_err(|error| error.to_string())?;
  result
    .recv()
    .map_err(|_| "The preview's audio output did not open".to_owned())??;
  Ok(Open {
    config,
    device: name,
    failed,
    slot,
    _close: close,
  })
}
