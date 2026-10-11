// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The camera's effects as a drawing applies them: what is behind the
//! person, the grade, the picture, and the matte to read beside the camera.
//! Loaded from the project whenever they change, and read by the preview,
//! the export, the thumbnails and the stills alike.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tauri::{AppHandle, Manager};

use super::grade::{self, Grade};
use super::{choice, image, make, matte, picks, CameraBackground, CameraRecording};
use crate::editor::EditorState;

static GENERATION: AtomicU64 = AtomicU64::new(1);

/// The matte file, and when each of its frames' camera frames shows.
#[derive(Clone, Debug)]
pub(crate) struct MatteFile {
  pub(crate) path: PathBuf,
  pub(crate) timings: Arc<screenwide_transcriber::MatteTimings>,
}

impl MatteFile {
  /// How long the matte file runs, in its own milliseconds.
  pub(crate) fn duration_ms(&self) -> u64 {
    (self.timings.frames_ms.len() as u64 * 1_000)
      .div_ceil(u64::from(self.timings.frames_per_second))
  }

  /// Where in the matte file the matte of the camera frame showing at
  /// camera time `ms` is: the start of that frame's slot, in its own
  /// milliseconds.
  pub(crate) fn matte_ms(&self, ms: u64) -> u64 {
    // A frame shows from its own time; the decoders hand over a frame
    // stamped up to 2 ms after the moment asked for, so the matte does too.
    let shown = self
      .timings
      .frames_ms
      .partition_point(|&frame_ms| frame_ms <= ms as f64 + 2.0)
      .saturating_sub(1);
    (shown as u64 * 1_000).div_ceil(u64::from(self.timings.frames_per_second))
  }
}

/// The effects one drawing applies to the camera.
pub(crate) struct LiveCameraEffects {
  pub(crate) background: CameraBackground,
  pub(crate) grade: Option<Grade>,
  pub(crate) picture: Option<Arc<::image::RgbaImage>>,
  pub(crate) matte: Option<MatteFile>,
  /// Sharpened, and enlarged along its edges where drawn larger.
  pub(crate) enhance: bool,
  /// Different for every load, so what is cached from one set of effects,
  /// such as the picture on the GPU, is not drawn with the next.
  pub(crate) generation: u64,
}

/// The camera's effects as its project has them now, or nothing where the
/// camera is drawn as recorded. A background whose matte is not made yet is
/// drawn as recorded until it is.
pub(crate) fn load(camera: &CameraRecording) -> Option<Arc<LiveCameraEffects>> {
  let folder = camera.folder();
  let effects = choice(&folder);
  let matte = effects
    .removes_background()
    .then(|| {
      matte::is_made(&folder)
        .then(|| matte::timings(&folder))
        .flatten()
    })
    .flatten()
    .map(|timings| MatteFile {
      path: picks::composed(&folder).unwrap_or_else(|| matte::path(&folder)),
      timings,
    });
  let background = if matte.is_some() {
    effects.background
  } else {
    CameraBackground::Kept
  };
  let picture = (background == CameraBackground::Image)
    .then(|| ::image::open(image::path(&folder)).ok())
    .flatten()
    .map(|picture| Arc::new(picture.to_rgba8()));
  // A picture that cannot be read leaves nothing to draw the person over.
  let background = if background == CameraBackground::Image && picture.is_none() {
    CameraBackground::Kept
  } else {
    background
  };
  let grade = effects
    .colour
    .then(|| make::measured(&folder))
    .flatten()
    .map(|measured| {
      let subject = measured.person.unwrap_or(measured.frame);
      match picture.as_deref().and_then(grade::measure_picture) {
        Some(picture) => grade::towards(&subject, &picture),
        None => grade::balanced(&measured.frame, &subject),
      }
    });
  if background == CameraBackground::Kept && grade.is_none() && !effects.enhance {
    return None;
  }
  Some(Arc::new(LiveCameraEffects {
    background,
    grade,
    picture,
    enhance: effects.enhance,
    matte,
    generation: GENERATION.fetch_add(1, Ordering::Relaxed),
  }))
}

/// The camera's effects for the recording open in the editor as
/// `artifact_id`.
pub(crate) fn for_recording(app: &AppHandle, artifact_id: u64) -> Option<Arc<LiveCameraEffects>> {
  let camera = {
    let state = app.state::<EditorState>();
    let artifact = state
      .recording
      .artifact
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let artifact = artifact.as_ref()?;
    match artifact {
      crate::editor::EditorArtifact::Recording { id, .. } if *id == artifact_id => {
        CameraRecording::of(artifact)?
      }
      _ => return None,
    }
  };
  load(&camera)
}
