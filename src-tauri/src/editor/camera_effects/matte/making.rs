// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Making the chunks of the matte the background ranges need, and joining
//! them into the whole matte once they cover the whole camera.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use screenwide_speech::runner::Transcriber;
use screenwide_transcriber::{Job, MatteChunk, MatteTimings};

use super::super::{picks, stop};
use super::chunks::{self, Span};
use super::{
  chunk_paths, chunks_folder, is_made, join, made_chunks, matte_size, missing, path, remove_chunks,
  saved, shape, timings_path, CameraRecording,
};

/// The chunks each camera's matte job is making, by the camera's folder.
static MAKING: Mutex<Vec<(PathBuf, Vec<Span>)>> = Mutex::new(Vec::new());

fn making() -> std::sync::MutexGuard<'static, Vec<(PathBuf, Vec<Span>)>> {
  MAKING
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The chunks a matte job is making, noted until it ends.
struct Making(PathBuf);

impl Making {
  fn begin(folder: &Path, chunks: &[Span]) -> Self {
    making().push((folder.to_owned(), chunks.to_vec()));
    Self(folder.to_owned())
  }
}

impl Drop for Making {
  fn drop(&mut self) {
    making().retain(|(folder, _)| *folder != self.0);
  }
}

/// Whether the matte job under way for `camera` is making a chunk that the
/// project's background ranges no longer cover any of, and that is not
/// made yet.
pub(in crate::editor::camera_effects) fn making_unneeded(camera: &CameraRecording) -> bool {
  let folder = camera.folder();
  let Some(chunks) = making()
    .iter()
    .find(|(at, _)| *at == folder)
    .map(|(_, chunks)| chunks.clone())
  else {
    return false;
  };
  let (ranges, timeline) = saved(camera);
  let needed = chunks::needed(&ranges, &timeline, camera.duration_ms);
  let made: Vec<Span> = made_chunks(&folder)
    .into_iter()
    .map(|(span, _)| span)
    .collect();
  chunks.iter().any(|&[from, to]| {
    let still_needed = needed.iter().any(|[start, end]| *start < to && from < *end);
    !still_needed && chunks::missing(&[[from, to]], &made) == [[from, to]]
  })
}

/// Makes the chunks of the matte of `camera` that its background ranges
/// need, nearest `playhead_ms` first, with the model at `model`, telling
/// `progress` how far it has got and `chunk_made` each time a chunk is
/// made. Once the chunks cover the whole camera, they are joined into the
/// whole matte, which picking objects reads.
pub(in crate::editor::camera_effects) fn prepare(
  camera: &CameraRecording,
  (model, playhead_ms): (PathBuf, u64),
  progress: &mut dyn FnMut(f32),
  chunk_made: &mut dyn FnMut(),
) -> Result<(), String> {
  let folder = camera.folder();
  let (missing, _) = missing(camera);
  if missing.is_empty() {
    return join_when_whole(camera);
  }
  let (size, frames_per_second) = shape(&camera.path);
  if size.0 == 0 || size.1 == 0 {
    return Err("The camera recording cannot be read".to_owned());
  }
  let (width, height) = matte_size(size);
  let order = chunks::nearest_first(chunks::chunks(&missing), playhead_ms);
  let chunks_folder = chunks_folder(&folder);
  std::fs::create_dir_all(&chunks_folder)
    .map_err(|error| format!("Could not keep the camera's matte: {error}"))?;
  remove_partials(&chunks_folder);
  let job_chunks = order
    .iter()
    .map(|&span @ [start_ms, end_ms]| {
      let (matte, timings) = chunk_paths(&folder, span);
      MatteChunk {
        start_ms,
        // A chunk that reaches the camera's end keeps every frame after it.
        end_ms: if end_ms >= camera.duration_ms {
          u64::MAX
        } else {
          end_ms
        },
        matte,
        timings,
      }
    })
    .collect::<Vec<_>>();
  let camera_ms: u64 = order.iter().map(|[start, end]| end - start).sum();
  let frames = (camera_ms as f64 * frames_per_second / 1_000.0).ceil() as u64;
  let running = stop::begin(&folder, stop::Work::Matte);
  let _making = Making::begin(&folder, &order);
  let made = Transcriber::start()?
    .stopped_by(running.stop())
    .matte_camera(
      Job::MatteCamera {
        model,
        ffmpeg: screenwide_sidecar::ffmpeg_path(),
        camera: camera.path.clone(),
        width,
        height,
        frames,
        frames_per_second: frames_per_second.round().max(1.0) as u32,
        chunks: job_chunks,
      },
      progress,
      &mut |_| chunk_made(),
    );
  if made.is_err() {
    remove_partials(&chunks_folder);
  }
  made.and_then(|()| join_when_whole(camera))
}

/// Removes what a run stopped part-way left of the chunk it was making.
fn remove_partials(chunks_folder: &Path) {
  if let Ok(entries) = std::fs::read_dir(chunks_folder) {
    for path in entries.flatten().map(|entry| entry.path()) {
      if path.to_string_lossy().contains(".partial.") {
        let _ = std::fs::remove_file(path);
      }
    }
  }
}

/// Joins the chunks into the whole matte once they cover the whole camera,
/// and lets the chunks go.
fn join_when_whole(camera: &CameraRecording) -> Result<(), String> {
  let folder = camera.folder();
  let made = made_chunks(&folder);
  let spans: Vec<Span> = made.iter().map(|(span, _)| *span).collect();
  if is_made(&folder) || !chunks::whole(&spans, camera.duration_ms) {
    return Ok(());
  }
  let mut frames_ms = Vec::new();
  let mut parts = Vec::new();
  let mut frames_per_second = 0;
  let mut reached = 0;
  for (span, timings) in made {
    // A chunk lying inside one taken already would play its frames twice.
    if span[0] < reached || timings.frames_ms.is_empty() {
      reached = reached.max(span[1]);
      continue;
    }
    reached = span[1];
    frames_per_second = timings.frames_per_second;
    frames_ms.extend(timings.frames_ms);
    parts.push(chunk_paths(&folder, span).0);
  }
  if parts.is_empty() {
    return Err("The camera recording has no frames to read".to_owned());
  }
  let partial = path(&folder).with_extension("partial.mp4");
  join::join(&parts, &partial)?;
  let failed = |error: std::io::Error| format!("Could not keep the camera's matte: {error}");
  let timings = serde_json::to_vec(&MatteTimings {
    format: MatteTimings::FORMAT,
    frames_per_second,
    frames_ms,
  })
  .map_err(|error| error.to_string())?;
  std::fs::rename(&partial, path(&folder)).map_err(failed)?;
  std::fs::write(timings_path(&folder), timings).map_err(failed)?;
  // The objects were laid into a matte before; they are laid into this one
  // afresh.
  picks::remove_composed(&folder, None);
  remove_chunks(&folder);
  Ok(())
}
