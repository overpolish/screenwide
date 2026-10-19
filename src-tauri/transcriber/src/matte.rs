// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Camera background removal: the person in a camera recording found by
//! Robust Video Matting (RVM, ByteDance, GPL-3.0), a recurrent model that
//! carries what it saw in one frame into the next, so the edge holds still
//! between frames where an image model's would shimmer.
//!
//! FFmpeg decodes the camera and encodes the matte, so the helper needs no
//! codec of its own. The camera is matted a chunk at a time, in the order the
//! app asks, each chunk into a matte of its own: one frame for each camera
//! frame in the chunk, in turn, and a list of when each shows, which the app
//! finds the matte of the frame it draws by.
//!
//! Every chunk but the first starts three seconds early and lets the model
//! see them without keeping them: with less, RVM's memory of the person is
//! not yet settled by the chunk's first kept frame, and it can take part of
//! the room for them there. A chunk's matte is the same whichever chunks
//! were matted before it, so the app can ask for them in any order.
//!
//! On Windows the model runs on the GPU through DirectML where it can, which
//! answers as the CPU does and twice as fast, and takes the chunks one at a
//! time. Otherwise several chunks are matted side by side, one model each on
//! a share of the CPU. Each chunk's frames are cleaned up on several threads
//! ([`in_order`]), which otherwise take longer than the model.

mod cleanup;
mod compose;
pub(crate) mod decode;
mod encode;
mod in_order;
mod model;
#[cfg(test)]
mod tests;
mod track;

pub use self::compose::compose_matte;
pub use self::track::{follow_objects, Clicked, Follow};

use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc;

use ort::session::Session;

use self::decode::Decoder;
use self::encode::Encoder;
use self::in_order::{clean_in_order, Matted};
use self::model::Matting;
use crate::session::{on_cpu_threads, on_directml};
use screenwide_transcriber::{MatteChunk, MatteTimings};

/// The feature's name in what the helper reports.
const WHAT: &str = "Background removal";
/// How much each chunk but the first sees before it keeps a frame.
const WARM_UP_MS: u64 = 3_000;
/// The most chunks matted side by side on the CPU.
const MOST_SIDE_BY_SIDE: usize = 4;
/// The most threads a chunk cleans its frames up on: four keep up with the
/// model on the GPU.
const MOST_CLEANERS: usize = 4;

/// The `chunks` of the camera at `camera`, each matted at `size` and written
/// at `frames_per_second` to its own matte and timings, by the model at
/// `model` with `ffmpeg` doing the decoding and encoding. Chunks are started
/// in the order given; `chunk_done` is told the place of each as its matte
/// is written, and `progress` how far the job has got out of about `frames`.
pub fn matte_camera(
  (model, ffmpeg): (&Path, &Path),
  camera: &Path,
  size: (u32, u32),
  (frames, frames_per_second): (u64, u32),
  chunks: &[MatteChunk],
  progress: &mut dyn FnMut(f32),
  chunk_done: &mut dyn FnMut(usize),
) -> Result<(), String> {
  if size.0 == 0 || size.1 == 0 || frames_per_second == 0 {
    return Err("The camera recording has no frames to read".to_owned());
  }
  if chunks.is_empty() {
    return Ok(());
  }
  let cores = std::thread::available_parallelism().map_or(4, usize::from);
  let mut gpu = on_directml(model, WHAT);
  let count = if gpu.is_some() {
    1
  } else {
    (cores / 3).clamp(1, MOST_SIDE_BY_SIDE).min(chunks.len())
  };
  let threads = (cores / count).max(1);
  // A chunk's model and its cleanup share its threads; on the GPU, the model
  // takes none of them.
  let cleaners = if gpu.is_some() {
    threads.saturating_sub(2)
  } else {
    threads / 4
  }
  .clamp(1, MOST_CLEANERS);
  let done = AtomicU64::new(0);
  let stop = AtomicBool::new(false);
  let next = AtomicUsize::new(0);
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
  let (finished, written) = mpsc::channel::<usize>();
  let results = std::thread::scope(|scope| {
    let running = (0..count)
      .map(|_| {
        let session = gpu.take();
        let (next, finished) = (&next, finished.clone());
        scope.spawn(move || job.run(chunks, next, session, &finished))
      })
      .collect::<Vec<_>>();
    drop(finished);
    loop {
      for index in written.try_iter() {
        chunk_done(index);
      }
      progress((done.load(Ordering::Relaxed) as f32 / frames.max(1) as f32).min(1.0));
      if running.iter().all(|thread| thread.is_finished()) {
        break;
      }
      std::thread::sleep(std::time::Duration::from_millis(100));
    }
    for index in written.try_iter() {
      chunk_done(index);
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
  results.into_iter().collect()
}

/// What every chunk is matted with.
struct Job<'a> {
  model: &'a Path,
  ffmpeg: &'a Path,
  camera: &'a Path,
  size: (u32, u32),
  frames_per_second: u32,
  /// The CPU threads a chunk's model runs on, where it runs on the CPU.
  model_threads: usize,
  /// The threads a chunk cleans its frames up on.
  cleaners: usize,
  done: &'a AtomicU64,
  /// Set when a chunk fails, so the others stop rather than finish for
  /// nothing.
  stop: &'a AtomicBool,
}

impl Job<'_> {
  /// Mattes the chunks of `chunks` not yet taken, taking each in turn from
  /// `next`, with `session` or on the CPU without one. Tells `finished` the
  /// place of each chunk once its matte is written.
  fn run(
    &self,
    chunks: &[MatteChunk],
    next: &AtomicUsize,
    session: Option<Session>,
    finished: &mpsc::Sender<usize>,
  ) -> Result<(), String> {
    let made = (|| {
      let (width, height) = (self.size.0 as usize, self.size.1 as usize);
      let session = match session {
        Some(session) => session,
        None => on_cpu_threads(self.model, WHAT, self.model_threads)?,
      };
      let mut model = Matting::new(session, (width, height));
      loop {
        if self.stop.load(Ordering::Relaxed) {
          return Ok(());
        }
        let index = next.fetch_add(1, Ordering::Relaxed);
        let Some(chunk) = chunks.get(index) else {
          return Ok(());
        };
        model.restart();
        self.matte(chunk, &mut model)?;
        let _ = finished.send(index);
      }
    })();
    if made.is_err() {
      self.stop.store(true, Ordering::Relaxed);
    }
    made
  }

  /// Mattes `chunk` with `model`, writing its matte and timings whole or
  /// not at all.
  fn matte(&self, chunk: &MatteChunk, model: &mut Matting) -> Result<(), String> {
    let (width, height) = (self.size.0 as usize, self.size.1 as usize);
    let partial = chunk.matte.with_extension("partial.mp4");
    let from_ms = chunk.start_ms.saturating_sub(WARM_UP_MS);
    let mut decoder = Decoder::start(self.ffmpeg, self.camera, from_ms, self.size)?;
    let encoder = Encoder::start(self.ffmpeg, self.size, self.frames_per_second, &partial)?;
    // The model reads the next frames while the last are cleaned up. Only
    // the chunk's own frames are cleaned up and kept.
    let (send, receive) = mpsc::sync_channel::<Matted>(self.cleaners * 2);
    let times = std::thread::scope(|scope| {
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
          if ms >= chunk.end_ms as f64 {
            break;
          }
          model.run(&rgb, (width, height), &mut alpha)?;
          // The first chunk keeps every frame; a later one keeps only its
          // own.
          if chunk.start_ms > 0 && ms < chunk.start_ms as f64 {
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
    });
    let kept = times.and_then(|frames_ms| keep(chunk, &partial, frames_ms, self.frames_per_second));
    if kept.is_err() {
      let _ = std::fs::remove_file(&partial);
    }
    kept
  }
}

/// The chunk's matte, written to `partial` with frames showing from
/// `frames_ms`, kept as `chunk`'s with its timings. A chunk past the end of
/// the camera has no frames and keeps no matte, only timings saying so.
fn keep(
  chunk: &MatteChunk,
  partial: &Path,
  frames_ms: Vec<f64>,
  frames_per_second: u32,
) -> Result<(), String> {
  let failed = |error: std::io::Error| format!("Could not write the camera's matte: {error}");
  if frames_ms.is_empty() {
    let _ = std::fs::remove_file(partial);
  } else {
    std::fs::rename(partial, &chunk.matte).map_err(failed)?;
  }
  let timings = serde_json::to_vec(&MatteTimings {
    format: MatteTimings::FORMAT,
    frames_per_second,
    frames_ms,
  })
  .map_err(|error| error.to_string())?;
  // The timings say the chunk is made, so they land last and whole.
  let partial_timings = chunk.timings.with_extension("partial.json");
  std::fs::write(&partial_timings, timings).map_err(failed)?;
  std::fs::rename(&partial_timings, &chunk.timings).map_err(failed)
}
