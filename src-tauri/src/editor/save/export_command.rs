// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// The file an export wrote, for the Export island to show as done.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ExportedFile {
  /// How large the file came out, in bytes.
  #[ts(type = "number")]
  pub bytes: u64,
  pub path: String,
}

#[tauri::command]
pub async fn save_export(
  app: AppHandle,
  window: tauri::WebviewWindow,
  file_stem: String,
  options: RecordingExportOptions,
) -> Result<Option<ExportedFile>, String> {
  let kind = kind_of_window(&window)?;
  let job_options = options.clone();
  let RecordingExportOptions {
    camera_compression,
    compression,
    delete_project_after_export,
    cursor_effects,
    keyboard_effects,
    recording_output,
    screenshot_format,
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
    matches!(&artifact, EditorArtifact::Recording { .. }).then(|| CompletedRecordingExport {
      choices: RecordingExportChoices::offered_by(&artifact, &job_options),
      keyboard: keyboard_effects,
      output: recording_output.clone(),
    });
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
  let progress_app = app.clone();
  let job_cancellation = Arc::clone(&cancelled);
  // The artifact travels back with an error. Saving used to take it before the
  // write and lose its source on a disk or mux failure, making Retry impossible
  // even though the recording remained on disk.
  let (result, artifact) = tauri::async_runtime::spawn_blocking(move || {
    // An export runs for minutes, often while its window is covered or the
    // app is out of sight, where it would otherwise be napped.
    let _activity = crate::user_work::UserActivity::begin_keeping_awake("Exporting");
    let result = (|| -> Result<Option<PathBuf>, String> {
      std::fs::create_dir_all(&writing).map_err(|error| error.to_string())?;

      match &artifact {
        EditorArtifact::Screenshot { items, .. } => {
          let path = unique_path(
            &writing,
            &stem,
            screenshot_format.extension(),
            &|candidate| candidate.exists(),
          );
          let composed = compose_screenshot_workspace(items, &screenshot_output)?;
          let bytes = match screenshot_format {
            ScreenshotFormat::Png => encode_png(&composed)?,
            ScreenshotFormat::Jpeg => encode_jpeg(&composed)?,
          };
          std::fs::write(&path, bytes).map_err(|error| error.to_string())?;
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
    unfinished => {
      restore(&app, kind, artifact);
      emit_snapshot(&app, kind);
      return unfinished.map(|_| None);
    }
  };

  store_export_directory(&app, kind, directory)?;
  remember_completed_export(
    &app,
    cursor_effects,
    recording_preference,
    screenshot_preference.map(|output| output.canvas),
  );
  // The editor stays as it is, showing the finished file on its Export
  // island, until the island is put away (`close_exported_editor`): only then
  // is the native player retired and the window hidden. A recording opened
  // while this one exported owns the window by then, and stays in it.
  *app
    .state::<EditorState>()
    .slot(kind)
    .exported
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(path.clone());
  // Only once the files are published and the editor has let go of it.
  if kind == EditorKind::Screenshot {
    if let Err(error) =
      remember_screenshot_delete_project_after_export(&app, delete_project_after_export)
    {
      eprintln!("Could not remember the screenshot export choice: {error}");
    }
  }
  if delete_project_after_export {
    if let Some(project) = artifact.project() {
      if let Err(error) = screenwide_project::delete(&app, project) {
        eprintln!("Could not delete the exported project: {error}");
      }
    }
  } else if let Some(project) = artifact.project() {
    // Unless the project was opened again while it exported, its folder
    // takes any name it was given in the editor.
    super::super::project_name::settle_name(&app, project);
  }
  if crate::settings::current(&app).open_location_after_export {
    if let Err(error) = location::reveal(&path) {
      eprintln!("Could not open the export location: {error}");
    }
  }
  let bytes = std::fs::metadata(&path).map_or(0, |metadata| metadata.len());
  Ok(Some(ExportedFile {
    bytes,
    path: path.to_string_lossy().into_owned(),
  }))
}

/// Puts back an artifact whose export did not finish, so it can be tried
/// again. A recording opened in its place meanwhile wins: the one that was
/// exporting is safe in its project.
fn restore(app: &AppHandle, kind: EditorKind, artifact: EditorArtifact) {
  let state = app.state::<EditorState>();
  let mut slot = state
    .slot(kind)
    .artifact
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  if slot.is_none() {
    *slot = Some(artifact);
  }
}
