// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[tauri::command]
pub async fn save_export(
  app: AppHandle,
  window: tauri::WebviewWindow,
  file_stem: String,
  options: RecordingExportOptions,
) -> Result<Option<PathBuf>, String> {
  let kind = kind_of_window(&window)?;
  let job_options = options.clone();
  let RecordingExportOptions {
    camera_compression,
    compression,
    cursor_effects,
    recording_output,
    screenshot_output,
    ..
  } = options;
  if compression > 4 || camera_compression > 4 {
    return Err("Compression must be between 0 and 4".to_owned());
  }
  let stem =
    sanitize_file_stem(&file_stem).ok_or_else(|| "That file name cannot be used".to_owned())?;
  let directory =
    current_directory(&app, kind).ok_or_else(|| "There is nowhere to save this".to_owned())?;
  let artifact = take_artifact(&app, kind).ok_or_else(|| "There is nothing to save".to_owned())?;
  let artifact_id = match &artifact {
    EditorArtifact::Screenshot { id, .. } | EditorArtifact::Recording { id, .. } => *id,
  };
  let screenshot_preference =
    matches!(&artifact, EditorArtifact::Screenshot { .. }).then(|| screenshot_output.clone());
  let recording_preference =
    matches!(&artifact, EditorArtifact::Recording { .. }).then(|| recording_output.clone());
  let cancelled = Arc::new(AtomicBool::new(false));
  {
    let state = app.state::<EditorState>();
    let mut active = state
      .slot(kind)
      .active_export
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    // Per workspace: a recording encode and a screenshot save are independent
    // jobs, and only a second save of the same workspace is a contradiction.
    if active.is_some() {
      *state
        .slot(kind)
        .artifact
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(artifact);
      drop(active);
      emit_snapshot(&app, kind);
      return Err("Another export is already running".to_owned());
    }
    *active = Some(ActiveExportJob {
      artifact_id,
      cancelled: Arc::clone(&cancelled),
    });
  }

  let writing = directory.clone();
  let export_app = app.clone();
  let progress_app = app.clone();
  let job_cancellation = Arc::clone(&cancelled);
  // The artifact travels back with an error. Saving used to take it before the
  // write and lose its source on a disk or mux failure, making Retry impossible
  // even though the recording remained on disk.
  let (result, artifact) = tauri::async_runtime::spawn_blocking(move || {
    let result = (|| -> Result<Option<PathBuf>, String> {
      std::fs::create_dir_all(&writing).map_err(|error| error.to_string())?;

      match &artifact {
        EditorArtifact::Screenshot { items, .. } => {
          let path = unique_path(&writing, &stem, SCREENSHOT_EXTENSION, &|candidate| {
            candidate.exists()
          });
          let composed = compose_screenshot_workspace(&export_app, items, &screenshot_output)?;
          std::fs::write(&path, encode_png(&composed)?).map_err(|error| error.to_string())?;
          Ok(Some(path))
        }
        EditorArtifact::Recording { .. } => export_job::save_recording_artifact(
          &artifact,
          &writing,
          &stem,
          &progress_app,
          &job_cancellation,
          job_options,
        ),
      }
    })();

    (result, artifact)
  })
  .await
  .map_err(|error| error.to_string())?;

  lifecycle::clear_active_export(&app, kind, artifact_id);

  let path = match result {
    Ok(Some(path)) => path,
    Ok(None) => {
      *app
        .state::<EditorState>()
        .slot(kind)
        .artifact
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(artifact);
      emit_snapshot(&app, kind);
      return Ok(None);
    }
    Err(error) => {
      *app
        .state::<EditorState>()
        .slot(kind)
        .artifact
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(artifact);
      emit_snapshot(&app, kind);
      return Err(error);
    }
  };

  if let EditorArtifact::Recording { path: working, .. } = &artifact {
    timeline_edit::remove_for_recording(working);
  }

  store_export_directory(&app, kind, directory)?;
  remember_completed_export(
    &app,
    cursor_effects,
    recording_preference,
    screenshot_preference.map(|output| output.canvas),
  );
  // Saving is transactional: keep the native player alive while the artifact
  // may still be restored by Cancel or an export error, then retire it only
  // once the finished files have been published.
  if kind == EditorKind::Recording {
    artifact::clear_recording_preview(&app);
  }
  let _ = window::hide(&app, kind);
  emit_snapshot(&app, kind);

  if crate::settings::current(&app).open_location_after_export {
    if let Err(error) = location::open_containing_folder(&path) {
      eprintln!("Could not open the export location: {error}");
    }
  }
  Ok(Some(path))
}
