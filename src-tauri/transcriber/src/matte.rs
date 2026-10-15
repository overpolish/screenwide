// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Camera background removal: the person in a camera recording found by
//! Robust Video Matting (RVM, ByteDance, GPL-3.0), a recurrent model that
//! carries what it saw in one frame into the next, so the edge holds still
//! between frames where an image model's would shimmer.
//!
//! FFmpeg decodes the camera and encodes the matte, so the helper needs no
//! codec of its own. The matte has one frame for each camera frame, in turn,
//! and a list of when each camera frame shows, which the app finds the matte
//! of the frame it draws by.
//!
//! On Windows the model runs on the GPU through DirectML where it can, which
//! answers as the CPU does and twice as fast, and the camera is matted in one
//! part. Otherwise the camera is matted in parts side by side, one model each
//! on a share of the CPU. Each part but the first starts three seconds early
//! and lets the model see them without keeping them: with less, RVM's memory
//! of the person is not yet settled by the part's first kept frame, and it
//! can take part of the room for them there.
//!
//! Each part's frames are cleaned up on several threads ([`in_order`]),
//! which otherwise take longer than the model.

mod cleanup;
mod compose;
mod decode;
mod encode;
mod in_order;
mod model;
#[cfg(test)]
mod tests;
mod track;

pub use self::compose::compose_matte;
pub use self::track::{follow_objects, Clicked, Follow};

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use ort::session::Session;

use self::decode::Decoder;
use self::encode::Encoder;
use self::in_order::{clean_in_order, Matted};
use self::model::Matting;
use crate::session::{on_cpu_threads, on_directml};
use screenwide_transcriber::MatteTimings;

/// The feature's name in what the helper reports.
const WHAT: &str = "Background removal";
/// How much each part but the first sees before it keeps a frame.
const WARM_UP_MS: u64 = 3_000;
/// How many warm-ups long a part is at the least, so warming up costs it no
/// more than a quarter again.
const PART_WARM_UPS: u64 = 4;
const MOST_PARTS: usize = 4;
/// The most threads a part cleans its frames up on: four keep up with the
/// model on the GPU.
const MOST_CLEANERS: usize = 4;

/// The camera at `camera` matted at `size`, written to `matte` at
/// `frames_per_second` with its frames' times in `timings`, by the model at
/// `model` with `ffmpeg` doing the decoding and encoding. Tells `progress`
/// how far it has got out of about `frames`.
#[allow(clippy::too_many_arguments)]
pub fn matte_camera(
  (model, ffmpeg): (&Path, &Path),
  camera: &Path,
  size: (u32, u32),
  (duration_ms, frames): (u64, u64),
  frames_per_second: u32,
  (matte, timings): (&Path, &Path),
  progress: &mut dyn FnMut(f32),
) -> Result<(), String> {
  if size.0 == 0 || size.1 == 0 || frames_per_second == 0 {
    return Err("The camera recording has no frames to read".to_owned());
  }
  let cores = std::thread::available_parallelism().map_or(4, usize::from);
  let mut gpu = on_directml(model, WHAT);
  let long_enough = (duration_ms / (WARM_UP_MS * PART_WARM_UPS)).max(1) as usize;
  let count = if gpu.is_some() {
    1
  } else {
    (cores / 3).clamp(1, MOST_PARTS).min(long_enough)
  };
  let threads = (cores / count).max(1);
  // A part's model and its cleanup share its threads; on the GPU, the model
  // takes none of them.
  let cleaners = if gpu.is_some() {
    threads.saturating_sub(2)
  } else {
    threads / 4
  }
  .clamp(1, MOST_CLEANERS);
  let parts = (0..count)
    .map(|index| Part {
      start_ms: duration_ms * index as u64 / count as u64,
      end_ms: if index + 1 == count {
        u64::MAX
      } else {
        duration_ms * (index as u64 + 1) / count as u64
      },
      path: matte.with_extension(format!("part{index}.mp4")),
    })
    .collect::<Vec<_>>();
  let done = AtomicU64::new(0);
  let stop = AtomicBool::new(false);
  let job = &Job {
    model,
    ffmpeg,
    camera,
    size,
    frames_per_second,
    model_threads: threads.saturating_sub(cleaners).max(1),
    cleaners,
    done: &done,
    stop: &stop,
  };
  let results = std::thread::scope(|scope| {
    let running = parts
      .iter()
      .map(|part| {
        let session = gpu.take();
        scope.spawn(move || job.run(part, session))
      })
      .collect::<Vec<_>>();
    while !running.iter().all(|thread| thread.is_finished()) {
      progress((done.load(Ordering::Relaxed) as f32 / frames.max(1) as f32).min(1.0));
      std::thread::sleep(std::time::Duration::from_millis(100));
    }
    running
      .into_iter()
      .map(|thread| {
        thread
          .join()
          .unwrap_or_else(|_| Err(format!("{WHAT} stopped unexpectedly")))
      })
      .collect::<Vec<_>>()
  });
  let finished = finish(ffmpeg, &parts, results, frames_per_second, (matte, timings));
  for part in &parts {
    let _ = std::fs::remove_file(&part.path);
  }
  finished
}

/// The share of the camera one model mattes: its frames from `start_ms`, up
/// to but not including `end_ms`.
struct Part {
  start_ms: u64,
  end_ms: u64,
  path: PathBuf,
}

/// What every part is matted with.
struct Job<'a> {
  model: &'a Path,
  ffmpeg: &'a Path,
  camera: &'a Path,
  size: (u32, u32),
  frames_per_second: u32,
  /// The CPU threads a part's model runs on, where it runs on the CPU.
  model_threads: usize,
  /// The threads a part cleans its frames up on.
  cleaners: usize,
  done: &'a AtomicU64,
  /// Set when a part fails, so the others stop rather than finish for
  /// nothing.
  stop: &'a AtomicBool,
}

impl Job<'_> {
  /// Mattes `part` into its own file with `session`, or on the CPU without
  /// one, answering its frames' times.
  fn run(&self, part: &Part, session: Option<Session>) -> Result<Vec<f64>, String> {
    let made = self.matte(part, session);
    if made.is_err() {
      self.stop.store(true, Ordering::Relaxed);
    }
    made
  }

  fn matte(&self, part: &Part, session: Option<Session>) -> Result<Vec<f64>, String> {
    let (width, height) = (self.size.0 as usize, self.size.1 as usize);
    let session = match session {
      Some(session) => session,
      None => on_cpu_threads(self.model, WHAT, self.model_threads)?,
    };
    let mut model = Matting::new(session, (width, height));
    let from_ms = part.start_ms.saturating_sub(WARM_UP_MS);
    let mut decoder = Decoder::start(self.ffmpeg, self.camera, from_ms, self.size)?;
    let encoder = Encoder::start(self.ffmpeg, self.size, self.frames_per_second, &part.path)?;
    // The model reads the next frames while the last are cleaned up. Only
    // the part's own frames are cleaned up and kept.
    let (send, receive) = std::sync::mpsc::sync_channel::<Matted>(self.cleaners * 2);
    std::thread::scope(|scope| {
      let writer = scope
        .spawn(move || clean_in_order(receive, encoder, (width, height), self.cleaners, self.done));
      let mut times = Vec::new();
      let read = (|| {
        let mut rgb = vec![0_u8; width * height * 3];
        let mut alpha = vec![0_u8; width * height];
        while let Some(ms) = decoder.next(&mut rgb)? {
          if self.stop.load(Ordering::Relaxed) {
            return Err(format!("{WHAT} stopped"));
          }
          if ms >= part.end_ms as f64 {
            break;
          }
          model.run(&rgb, (width, height), &mut alpha)?;
          // The first part keeps every frame; a later one keeps only its own.
          if part.start_ms > 0 && ms < part.start_ms as f64 {
            continue;
          }
          let index = times.len() as u64;
          if send.send((index, alpha.clone(), rgb.clone())).is_err() {
            break;
          }
          times.push(ms);
        }
        Ok(())
      })();
      drop(send);
      let written = writer
        .join()
        .unwrap_or_else(|_| Err(format!("{WHAT} stopped unexpectedly")));
      read.and(written).map(|()| times)
    })
  }
}

/// The parts joined, in turn, into `matte`, and their frames' times written
/// to `timings`.
fn finish(
  ffmpeg: &Path,
  parts: &[Part],
  results: Vec<Result<Vec<f64>, String>>,
  frames_per_second: u32,
  (matte, timings): (&Path, &Path),
) -> Result<(), String> {
  let mut frames_ms = Vec::new();
  let mut kept = Vec::new();
  for (part, result) in parts.iter().zip(results) {
    let times = result?;
    if !times.is_empty() {
      frames_ms.extend(times);
      kept.push(part.path.as_path());
    }
  }
  if frames_ms.is_empty() {
    return Err("The camera recording has no frames to read".to_owned());
  }
  join(ffmpeg, &kept, matte)?;
  let timings_json = serde_json::to_vec(&MatteTimings {
    format: MatteTimings::FORMAT,
    frames_per_second,
    frames_ms,
  })
  .map_err(|error| error.to_string())?;
  std::fs::write(timings, timings_json)
    .map_err(|error| format!("Could not write the camera's matte: {error}"))
}

/// The parts at `parts`, made alike, one after the other into `output`
/// without encoding them again.
fn join(ffmpeg: &Path, parts: &[&Path], output: &Path) -> Result<(), String> {
  if let [only] = parts {
    return std::fs::copy(only, output)
      .map(|_| ())
      .map_err(|error| format!("Could not write the camera's matte: {error}"));
  }
  let list = output.with_extension("parts.txt");
  let listed = parts
    .iter()
    .map(|part| {
      format!(
        "file '{}'\n",
        part.display().to_string().replace('\'', "'\\''")
      )
    })
    .collect::<String>();
  std::fs::write(&list, listed)
    .map_err(|error| format!("Could not write the camera's matte: {error}"))?;
  let status = Command::new(ffmpeg)
    .args(["-hide_banner", "-nostdin", "-loglevel", "error"])
    .args(["-f", "concat", "-safe", "0", "-i"])
    .arg(&list)
    .args(["-c", "copy", "-movflags", "+faststart", "-y"])
    .arg(output)
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status();
  let _ = std::fs::remove_file(&list);
  match status {
    Ok(status) if status.success() => Ok(()),
    Ok(_) => Err("FFmpeg could not join the camera's matte".to_owned()),
    Err(error) => Err(format!("Could not join the camera's matte: {error}")),
  }
}
