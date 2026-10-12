// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The matte: how much of each pixel of the camera is the person, made from
//! the recording by Robust Video Matting in the transcriber program. It has
//! one frame for each camera frame, in turn, beside a list of when each
//! camera frame shows, so the matte of the frame on screen is always the
//! matte of that frame.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use screenwide_transcriber::{Job, MatteTimings};
use tauri::AppHandle;

use super::CameraRecording;
use crate::transcription::models;
use screenwide_speech::catalogue::{self, Purpose};
use screenwide_speech::runner::Transcriber;

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

/// Makes the matte of `camera` into its project with the model at `model`,
/// unless it is there already, telling `progress` how far it has got.
pub(super) fn prepare(
  camera: &CameraRecording,
  model: PathBuf,
  progress: &mut dyn FnMut(f32),
) -> Result<(), String> {
  let folder = camera.folder();
  if is_made(&folder) {
    return Ok(());
  }
  let (size, frames_per_second) = shape(&camera.path);
  if size.0 == 0 || size.1 == 0 {
    return Err("The camera recording cannot be read".to_owned());
  }
  let (width, height) = matte_size(size);
  let destination = path(&folder);
  let partial = destination.with_extension("partial.mp4");
  let partial_timings = timings_path(&folder).with_extension("partial.json");
  let frames = (camera.duration_ms as f64 * frames_per_second / 1_000.0).ceil() as u64;
  let running = super::stop::begin(&folder, super::stop::Work::Matte);
  let made = Transcriber::start()?
    .stopped_by(running.stop())
    .matte_camera(
      Job::MatteCamera {
        model,
        ffmpeg: screenwide_sidecar::ffmpeg_path(),
        camera: camera.path.clone(),
        width,
        height,
        duration_ms: camera.duration_ms,
        frames,
        frames_per_second: frames_per_second.round().max(1.0) as u32,
        matte: partial.clone(),
        timings: partial_timings.clone(),
      },
      progress,
    );
  let kept = made.and_then(|()| {
    std::fs::rename(&partial, &destination)
      .and_then(|()| std::fs::rename(&partial_timings, timings_path(&folder)))
      .map_err(|error| format!("Could not keep the camera's matte: {error}"))
  });
  if kept.is_err() {
    // Besides the matte and its timings, the parts it is made in, which a
    // run stopped part-way leaves.
    let unfinished = partial.with_extension("");
    let unfinished = format!("{}.", unfinished.to_string_lossy());
    if let Ok(entries) = std::fs::read_dir(&folder) {
      for path in entries.flatten().map(|entry| entry.path()) {
        if path.to_string_lossy().starts_with(&unfinished) {
          let _ = std::fs::remove_file(path);
        }
      }
    }
  } else {
    // The objects were laid into the matte before; they are laid into this
    // one afresh.
    super::picks::remove_composed(&folder, None);
  }
  kept
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
