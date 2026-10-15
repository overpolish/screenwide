// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Loading the camera's effects from the project whenever they change, for
//! the preview, the export, the thumbnails and the stills.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tauri::{AppHandle, Manager};

use super::{choice, image, make, matte, CameraRecording};
use crate::editor::EditorState;
use screenwide_export::camera_effects::grade::Grade;
use screenwide_export::camera_effects::live::{LiveBackground, LiveCameraEffects};
use screenwide_export::camera_effects::CameraBackground;
use screenwide_timeline_edit::{
  CameraBackgroundRange, CameraRangeBackground, RecordingTimelineEdit, TimelinePlan,
};

static GENERATION: AtomicU64 = AtomicU64::new(1);

/// The camera's effects with the background ranges of `edit`, or of the
/// edit the project keeps where none is given; nothing where the camera is
/// drawn as recorded. Ranges whose matte is not made yet, or whose picture
/// cannot be read, are drawn as recorded until they can be.
pub(crate) fn load(
  camera: &CameraRecording,
  edit: Option<&RecordingTimelineEdit>,
) -> Option<Arc<LiveCameraEffects>> {
  let folder = camera.folder();
  let effects = choice(&folder);
  let saved;
  let edit = match edit {
    Some(edit) => Some(edit),
    None => {
      saved = crate::editor::timeline_edit::for_project(&camera.project, 0, camera.recording_ms);
      saved.as_ref().map(|(_, edit)| edit)
    }
  };
  let ranges = edit.map_or(&[][..], |edit| &edit.camera_background_ranges[..]);
  let matte = if ranges.is_empty() {
    Vec::new()
  } else {
    matte::readable(camera)
  };
  let backgrounds = if matte.is_empty() {
    Vec::new()
  } else {
    backgrounds(&folder, ranges)
  };
  let grade = effects
    .enhance
    .then(|| make::measured(&folder))
    .flatten()
    .map(|measured| Grade::new(measured.gains));
  if backgrounds.is_empty() && grade.is_none() && !effects.enhance {
    return None;
  }
  let plan = edit.and_then(|edit| TimelinePlan::from_edit(edit, camera.recording_ms));
  Some(Arc::new(LiveCameraEffects {
    backgrounds,
    timeline: plan
      .as_ref()
      .map_or_else(Vec::new, |plan| plan.ranges().to_vec()),
    video_ms: plan.map_or(camera.recording_ms, |plan| plan.duration_ms()),
    grade,
    enhance: effects.enhance,
    matte,
    generation: GENERATION.fetch_add(1, Ordering::Relaxed),
  }))
}

/// `ranges` as the drawing reads them, each with its picture, leaving out a
/// picture range whose picture cannot be read: nothing would be behind the
/// person there.
fn backgrounds(folder: &std::path::Path, ranges: &[CameraBackgroundRange]) -> Vec<LiveBackground> {
  let pictured: Vec<_> = ranges
    .iter()
    .filter(|range| range.background == CameraRangeBackground::Image)
    .map(|range| image::path(folder, &range.id))
    .collect();
  let mut pictures = image::decoded(&pictured).into_iter();
  ranges
    .iter()
    .filter_map(|range| {
      let (background, picture) = match range.background {
        CameraRangeBackground::Transparent => (CameraBackground::Transparent, None),
        CameraRangeBackground::Blurred => (CameraBackground::Blurred, None),
        CameraRangeBackground::Image => (CameraBackground::Image, Some(pictures.next()??)),
      };
      Some(LiveBackground {
        start_ms: range.start_ms,
        end_ms: range.end_ms,
        background,
        picture,
      })
    })
    .collect()
}

/// The camera's effects for the recording open in the editor as
/// `artifact_id`, with the ranges of the edit its project keeps.
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
  load(&camera, None)
}
