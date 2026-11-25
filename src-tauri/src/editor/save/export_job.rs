// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

mod gif;
mod prepared;
mod publish;
mod subtitled;
pub(super) use prepared::{prepare, Prepared, Route};
pub(super) use subtitled::{captioned, save_recording_artifact, write_subtitles, Subtitles};

/// Exports the recording's media: its video, camera and audio as `options`
/// ask; [`save_recording_artifact`] adds its subtitles.
fn save_recording_media(
  artifact: &EditorArtifact,
  writing: &Path,
  stem: &str,
  progress_app: &AppHandle,
  job_cancellation: &AtomicBool,
  options: RecordingExportOptions,
) -> Result<Option<PathBuf>, String> {
  let prepared = prepare(artifact, progress_app, options)?;
  let EditorArtifact::Recording {
    camera,
    id,
    path: working,
    ..
  } = artifact
  else {
    unreachable!("only a recording is prepared");
  };
  let options = &prepared.options;
  let duration_ms = prepared.duration_ms(artifact);
  match prepared.route {
    Route::Sound => audio_save::save_audio(audio_save::AudioSaveRequest {
      app: progress_app,
      cancelled: job_cancellation,
      directory: writing,
      duration_ms,
      file: options.recording_format.sound_file(),
      id: *id,
      layout: prepared.layout,
      selected_any: !options.enabled_stream_indices.is_empty(),
      selection: &prepared.selection,
      stem,
      timeline: prepared.timeline.as_ref(),
      working,
    }),
    Route::CameraAlone => camera_save::save_camera_as_primary(
      working,
      camera
        .as_ref()
        .ok_or_else(|| "There is no camera track to export".to_owned())?,
      writing,
      stem,
      &prepared.selection,
      prepared.layout,
      *id,
      progress_app,
      job_cancellation,
      options.camera_compression,
      options.camera_resolution_scale_percent,
      &options.recording_output.camera,
      prepared.timeline.as_ref(),
      prepared.camera_effects.clone(),
    ),
    Route::Baked => camera_save::save_baked_recording(
      prepared
        .drawn(artifact)
        .ok_or_else(|| "There is no camera recording to bake in".to_owned())?,
      duration_ms,
      writing,
      stem,
      *id,
      progress_app,
      job_cancellation,
    ),
    Route::Screen => {
      let share = screen_progress_share(&prepared, artifact);
      let saved = save_primary_recording(PrimaryRecordingSaveRequest {
        app: progress_app,
        artifact_id: *id,
        audio_tracks: match artifact {
          EditorArtifact::Recording { audio_tracks, .. } => audio_tracks,
          EditorArtifact::Screenshot { .. } => &[],
        },
        cancelled: job_cancellation,
        directory: writing,
        picture: prepared.screen_alone(artifact),
        progress_share: share,
        stem,
      })?;
      let Some(saved) = saved else {
        return Ok(None);
      };
      finish_screen(
        &prepared,
        artifact,
        saved,
        writing,
        progress_app,
        job_cancellation,
        (Some(stem), (share, 99.0 - share)),
      )
    }
  }
}

/// How much of the progress bar the drawn video takes: half where the
/// camera follows as a file of its own, which a baked camera never does.
pub(super) fn screen_progress_share(prepared: &Prepared, artifact: &EditorArtifact) -> f64 {
  let keeps_camera = matches!(prepared.route, Route::Screen)
    && matches!(
      artifact,
      EditorArtifact::Recording {
        camera: Some(_),
        ..
      }
    )
    && prepared.options.include_camera;
  if keeps_camera {
    50.0
  } else {
    99.0
  }
}

/// Finishes a screen's export whose video is written at `saved`: the camera
/// as a file of its own beside it, where it is kept, named after
/// `camera_stem` and reporting from `start` over `share` of the progress
/// bar, and both confirmed. With no `camera_stem` the camera is another
/// file's to write, as it is for every format but the first of an export
/// of every format.
pub(super) fn finish_screen(
  prepared: &Prepared,
  artifact: &EditorArtifact,
  saved: PathBuf,
  writing: &Path,
  progress_app: &AppHandle,
  job_cancellation: &AtomicBool,
  (camera_stem, (start, share)): (Option<&str>, (f64, f64)),
) -> Result<Option<PathBuf>, String> {
  let EditorArtifact::Recording { camera, id, .. } = artifact else {
    unreachable!("only a recording is prepared");
  };
  let options = &prepared.options;
  let camera_stem = camera_stem.filter(|_| options.include_camera);
  let next_phase = if camera_stem.is_some() {
    "camera"
  } else {
    "finalizing"
  };
  let _ = progress_app.emit(
    EXPORT_PROGRESS_EVENT,
    ExportProgress {
      artifact_id: *id,
      phase: next_phase,
      progress_percent: start,
    },
  );
  let mut saved_camera = None;
  if let Some(camera_stem) = camera_stem {
    let camera = camera
      .as_ref()
      .ok_or_else(|| "There is no camera track to export".to_owned())?;
    let camera_path = camera_save::save_camera_copy(
      camera,
      writing,
      camera_stem,
      *id,
      progress_app,
      job_cancellation,
      (start, share),
      options.camera_compression,
      options.camera_resolution_scale_percent,
      // Saved as a file of its own, the camera's canvas is a screen colour,
      // which is what shows where its background is taken away.
      &options.camera_key.keyed(&options.recording_output.camera),
      prepared.timeline.as_ref(),
      prepared.camera_effects.clone(),
    )
    .inspect_err(|_| {
      let _ = std::fs::remove_file(&saved);
    })?;
    let Some(camera_path) = camera_path else {
      let _ = std::fs::remove_file(&saved);
      return Ok(None);
    };
    saved_camera = Some(camera_path);
  }
  let _ = progress_app.emit(
    EXPORT_PROGRESS_EVENT,
    ExportProgress {
      artifact_id: *id,
      phase: "finalizing",
      progress_percent: start + share,
    },
  );
  publish::finish(&saved, saved_camera)?;
  Ok(Some(saved))
}
