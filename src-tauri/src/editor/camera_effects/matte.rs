// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The matte: how much of each pixel of the camera is the person, made from
//! the recording by Robust Video Matting in the transcriber program. It has
//! one frame for each camera frame, in turn, beside a list of when each
//! camera frame shows, so the matte of the frame on screen is always the
//! matte of that frame.
//!
//! It is made in chunks (`chunks`), only over the camera the background
//! ranges cover, and drawn from the chunks made so far. Once the chunks
//! cover the whole camera they are joined into the whole matte, which
//! picking objects reads.

mod chunks;
mod join;
mod making;

pub(super) use making::{making_unneeded, prepare};

use std::path::{Path, PathBuf};

use self::chunks::Span;
use std::sync::Arc;

use screenwide_export::camera_effects::live::MatteFile;
use screenwide_timeline::TimelineRange;
use screenwide_timeline_edit::{CameraBackgroundRange, TimelinePlan};
use screenwide_transcriber::MatteTimings;
use tauri::AppHandle;

use super::CameraRecording;
use crate::transcription::models;
use screenwide_speech::catalogue::{self, Purpose};

/// The rate a camera is taken to run at when it does not say its own.
const FALLBACK_FRAMES_PER_SECOND: f64 = 30.0;
/// The longest side the matte is made at. It is stretched over the camera
/// when drawn; its edge is soft, and finding the person happens smaller
/// still, so a larger matte only takes longer to make.
const LONGEST_SIDE: u32 = 960;

pub(crate) fn path(folder: &Path) -> PathBuf {
  folder.join("camera-matte.mp4")
}

pub(crate) fn timings_path(folder: &Path) -> PathBuf {
  folder.join("camera-matte.json")
}

/// When each of the matte's frames shows, where the matte is made in the
/// form this version reads.
pub(crate) fn timings(folder: &Path) -> Option<Arc<MatteTimings>> {
  let bytes = std::fs::read(timings_path(folder)).ok()?;
  serde_json::from_slice::<MatteTimings>(&bytes)
    .ok()
    .filter(|timings| {
      timings.format == MatteTimings::FORMAT
        && timings.frames_per_second > 0
        && !timings.frames_ms.is_empty()
    })
    .map(Arc::new)
}

pub(crate) fn is_made(folder: &Path) -> bool {
  path(folder).is_file() && timings(folder).is_some()
}

/// The matting model, where it is downloaded.
pub(crate) fn model(app: &AppHandle) -> Option<PathBuf> {
  let model = catalogue::for_purpose(Purpose::CameraBackground)?;
  if !models::is_downloaded(app, model) {
    return None;
  }
  models::paths(app, model).ok()?.into_iter().next()
}

/// The camera's size and the rate it says it was recorded at.
pub(crate) fn shape(camera: &Path) -> ((u32, u32), f64) {
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  let info = screenwide_export::media_preview::recording_info(camera);
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  let (size, rate) = (
    info.map_or((0, 0), |info| (info.width, info.height)),
    info.and_then(|info| info.frames_per_second),
  );
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  let (size, rate) = {
    let _ = camera;
    ((0, 0), None)
  };
  let rate = rate
    .filter(|rate| rate.is_finite() && *rate > 0.0)
    .unwrap_or(FALLBACK_FRAMES_PER_SECOND);
  (size, rate)
}

/// The size the matte of a camera `size` is made at: no longer than
/// [`LONGEST_SIDE`], in even pixels, as H.264 takes.
pub(super) fn matte_size((width, height): (u32, u32)) -> (u32, u32) {
  let factor = (f64::from(LONGEST_SIDE) / f64::from(width.max(height).max(1))).min(1.0);
  let even = |side: u32| ((f64::from(side) * factor).round() as u32).max(2) & !1;
  (even(width), even(height))
}

/// The folder the chunks of the matte are kept in until they cover the
/// whole camera, when they are joined into the whole matte.
fn chunks_folder(folder: &Path) -> PathBuf {
  folder.join("camera-matte-chunks")
}

/// The matte and timings of the chunk holding the camera's `[start, end)`.
fn chunk_paths(folder: &Path, [start, end]: Span) -> (PathBuf, PathBuf) {
  let chunks = chunks_folder(folder);
  (
    chunks.join(format!("{start}-{end}.mp4")),
    chunks.join(format!("{start}-{end}.json")),
  )
}

fn read_timings(path: &Path) -> Option<MatteTimings> {
  let bytes = std::fs::read(path).ok()?;
  serde_json::from_slice::<MatteTimings>(&bytes)
    .ok()
    .filter(|timings| timings.format == MatteTimings::FORMAT && timings.frames_per_second > 0)
}

/// The chunks of the matte made, in order, each with the camera it holds
/// and its timings. A chunk holding none of the camera's frames is made
/// with none.
fn made_chunks(folder: &Path) -> Vec<(Span, MatteTimings)> {
  let Ok(entries) = std::fs::read_dir(chunks_folder(folder)) else {
    return Vec::new();
  };
  let mut made: Vec<(Span, MatteTimings)> = entries
    .flatten()
    .filter_map(|entry| {
      let name = entry.file_name();
      let (start, end) = name.to_str()?.strip_suffix(".json")?.split_once('-')?;
      let span = [start.parse().ok()?, end.parse().ok()?];
      let timings = read_timings(&entry.path())?;
      (timings.frames_ms.is_empty() || chunk_paths(folder, span).0.is_file())
        .then_some((span, timings))
    })
    .collect();
  made.sort_unstable_by_key(|(span, _)| *span);
  made
}

/// The camera a chunk holds as its matte file says: to the camera's end,
/// and whatever frames lie past it, where it reaches the end.
fn covers([start, end]: Span, camera_ms: u64) -> (u64, u64) {
  (start, if end >= camera_ms { u64::MAX } else { end })
}

/// What the drawings read the matte from: the matte with the objects laid
/// in, the whole matte, or the chunks made of it so far, in order. Nothing
/// covers the camera where no chunk is made yet, so it is drawn as recorded
/// there.
pub(super) fn readable(camera: &CameraRecording) -> Vec<MatteFile> {
  let folder = camera.folder();
  if let Some(timings) = timings(&folder) {
    let path = super::picks::composed(&folder).unwrap_or_else(|| path(&folder));
    return vec![MatteFile {
      path,
      timings,
      covers: (0, u64::MAX),
    }];
  }
  made_chunks(&folder)
    .into_iter()
    .filter(|(_, timings)| !timings.frames_ms.is_empty())
    .map(|(span, timings)| MatteFile {
      path: chunk_paths(&folder, span).0,
      timings: Arc::new(timings),
      covers: covers(span, camera.duration_ms),
    })
    .collect()
}

/// The edit the project keeps for `camera`, and the timeline it lays out:
/// empty where the recording plays whole.
fn saved(camera: &CameraRecording) -> (Vec<CameraBackgroundRange>, Vec<TimelineRange>) {
  let Some((_, edit)) =
    crate::editor::timeline_edit::for_project(&camera.project, 0, camera.recording_ms)
  else {
    return (Vec::new(), Vec::new());
  };
  let timeline = TimelinePlan::from_edit(&edit, camera.recording_ms)
    .map_or_else(Vec::new, |plan| plan.ranges().to_vec());
  (edit.camera_background_ranges, timeline)
}

/// The camera the project's background ranges cover whose matte is not
/// made yet, and the timeline the ranges are timed on. None once the whole
/// matte is made.
fn missing(camera: &CameraRecording) -> (Vec<Span>, Vec<TimelineRange>) {
  let (ranges, timeline) = saved(camera);
  let folder = camera.folder();
  if is_made(&folder) {
    return (Vec::new(), timeline);
  }
  let made: Vec<Span> = made_chunks(&folder)
    .into_iter()
    .map(|(span, _)| span)
    .collect();
  let needed = chunks::needed(&ranges, &timeline, camera.duration_ms);
  (chunks::missing(&needed, &made), timeline)
}

/// Whether the project's background ranges cover camera whose matte is not
/// made yet.
pub(super) fn has_missing(camera: &CameraRecording) -> bool {
  !missing(camera).0.is_empty()
}

/// The stretches of the edited video the project's background ranges cover
/// whose matte is not made yet, in milliseconds.
pub(super) fn pending(camera: &CameraRecording) -> Vec<Span> {
  let (missing, timeline) = missing(camera);
  chunks::timeline_spans(&missing, &timeline)
}

/// Lets go of every chunk of the matte.
pub(super) fn remove_chunks(folder: &Path) {
  let _ = std::fs::remove_dir_all(chunks_folder(folder));
}

#[cfg(test)]
mod tests {
  use super::matte_size;

  #[test]
  fn the_matte_is_made_no_longer_than_its_longest_side_in_even_pixels() {
    assert_eq!(matte_size((1920, 1080)), (960, 540));
    assert_eq!(matte_size((1080, 1920)), (540, 960));
    assert_eq!(matte_size((640, 361)), (640, 360));
  }
}
