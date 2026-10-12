// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Loading the camera's effects from the project whenever they change, for
//! the preview, the export, the thumbnails and the stills.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tauri::{AppHandle, Manager};

use super::{choice, image, make, matte, picks, CameraRecording};
use crate::editor::EditorState;
use screenwide_export::camera_effects::grade::Grade;
use screenwide_export::camera_effects::live::{LiveCameraEffects, MatteFile};
use screenwide_export::camera_effects::CameraBackground;

static GENERATION: AtomicU64 = AtomicU64::new(1);

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
    .enhance
    .then(|| make::measured(&folder))
    .flatten()
    .map(|measured| Grade::new(measured.gains));
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
