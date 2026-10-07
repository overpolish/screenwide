// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Opening a project from disk into the recording editor.

use super::artifact_present::{present_recording_from, Origin};
use super::recording_sidecar::{valid_cursor, valid_keyboard};
use super::*;

/// Asks for a project and opens it.
pub fn choose_and_open_project(app: &AppHandle) {
  let app = app.clone();
  tauri::async_runtime::spawn_blocking(move || {
    use tauri_plugin_dialog::DialogExt;
    let mut dialog = app
      .dialog()
      .file()
      .set_title("Open Project")
      .add_filter("Screenwide Project", &[crate::project::EXTENSION]);
    if let Ok(directory) = crate::project::projects_directory(&app) {
      dialog = dialog.set_directory(directory);
    }
    let Some(file) = dialog
      .blocking_pick_file()
      .and_then(|path| path.into_path().ok())
    else {
      return;
    };
    open_and_report(&app, &file);
  });
}

/// Opens `file` off the calling thread: for opens asked for from outside the
/// app, by Finder, Explorer or a second launch.
pub fn open_project_detached(app: &AppHandle, file: PathBuf) {
  let app = app.clone();
  tauri::async_runtime::spawn_blocking(move || open_and_report(&app, &file));
}

/// The project a launch was asked to open: the last argument naming a
/// `.screenwide` file, with a relative one taken from `cwd`. Windows passes
/// the double-clicked file this way; macOS sends an open event instead.
pub fn project_argument<S: AsRef<std::ffi::OsStr>>(
  args: impl IntoIterator<Item = S>,
  cwd: &Path,
) -> Option<PathBuf> {
  args
    .into_iter()
    .map(|arg| PathBuf::from(arg.as_ref()))
    .filter(|path| is_project_file(path))
    .last()
    .map(|path| cwd.join(path))
}

pub fn is_project_file(path: &Path) -> bool {
  path
    .extension()
    .is_some_and(|extension| extension.eq_ignore_ascii_case(crate::project::EXTENSION))
}

/// A project that cannot be opened says why in an alert, since nothing else
/// is on screen to say it.
fn open_and_report(app: &AppHandle, file: &Path) {
  if let Err(error) = open_project(app, file) {
    crate::alert::show(app, "Project could not open", &error);
  }
}

/// Opens the project whose manifest is `file` in the editor its kind uses, in
/// place of whatever that editor had open.
pub fn open_project(app: &AppHandle, file: &Path) -> Result<(), String> {
  if crate::project::is_cancelled(file) {
    return Err("This recording was discarded".to_owned());
  }
  if let Some(kind) = open_kind(app, file) {
    workspace::focus_pending(app, kind);
    return Ok(());
  }
  // A name given while it was last open, which the app stopped before
  // giving its folder, is given now.
  let settled = super::project_name::settle_name(app, file);
  let file = settled.as_path();
  let manifest = crate::project::read(file)?;
  if manifest.kind == crate::project::ProjectKind::Screenshot {
    return super::screenshot_project::open(app, file);
  }
  let crate::project::RecordingManifest {
    duration_ms: _,
    has_microphone,
    has_system_audio,
    media,
    primary_kind,
    source_scale_factor,
    origin: _,
  } = manifest.recorded()?.clone();
  let primary = crate::project::resolve(file, &media.primary)?;
  if !primary.is_file() {
    return Err(format!(
      "The recording is missing from this project. Expected it at {}",
      primary.display()
    ));
  }
  // Before anything draws, so the project's images find their pictures on a
  // computer that has never stored them. Then tidied, in case it was last
  // closed by the app quitting, which leaves no chance to.
  super::project_look::adopt_pictures(file);
  super::project_look::clean_pictures(file);
  // A track that is gone, or that a crash cut off before it held anything
  // readable, leaves the recording without it rather than refusing it.
  let optional = |name: Option<String>| -> Result<Option<PathBuf>, String> {
    Ok(
      name
        .map(|name| crate::project::resolve(file, &name))
        .transpose()?
        .filter(|path| path.is_file()),
    )
  };
  let camera_path = optional(media.camera)?;
  let cursor_path = optional(media.cursor)?.and_then(valid_cursor);
  let keyboard_path = optional(media.keyboard)?.and_then(valid_keyboard);

  #[cfg(any(target_os = "macos", target_os = "windows"))]
  let primary_info = media_preview::recording_info(&primary);
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  if primary_kind == PrimaryRecordingKind::Screen
    && primary_info.is_none_or(|info| info.width == 0 || info.height == 0)
  {
    // Without readable dimensions there is no layout to preview it in.
    return Err(
      "The recording in this project cannot be played. It may have stopped before any picture was saved."
        .to_owned(),
    );
  }
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  let (duration_ms, width, height) = primary_info.map_or((0, 0, 0), |info| {
    (info.duration_ms, info.width, info.height)
  });
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  let camera = camera_path.map(|path| {
    let info = media_preview::recording_info(&path);
    crate::recording::CameraFinalizeInfo {
      duration_ms: info.map_or(0, |value| value.duration_ms),
      height: info.map_or(0, |value| value.height),
      path,
      width: info.map_or(0, |value| value.width),
    }
  });
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  let (duration_ms, width, height) = (0, 0, 0);
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  let camera = camera_path.map(|path| crate::recording::CameraFinalizeInfo {
    duration_ms: 0,
    height: 0,
    path,
    width: 0,
  });

  present_recording_from(
    app,
    file.to_path_buf(),
    FinalizeInfo {
      // Annotations drawn live were put in the project's edit when the
      // recording finished.
      annotation_clips: Vec::new(),
      camera,
      cursor_path,
      keyboard_path,
      has_microphone,
      has_system_audio,
      duration_ms,
      height,
      path: primary,
      primary_kind,
      // A nonsense factor would size the export list off a division by
      // something impossible.
      source_scale_factor: if source_scale_factor.is_finite() && source_scale_factor > 0.0 {
        source_scale_factor
      } else {
        1.0
      },
      width,
    },
    Origin::Project,
  )
}

/// The editor that has the project whose manifest is `file` open, if one does.
pub fn open_kind(app: &AppHandle, file: &Path) -> Option<EditorKind> {
  let state = app.state::<EditorState>();
  EditorKind::ALL.into_iter().find(|kind| {
    state
      .slot(*kind)
      .artifact
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .as_ref()
      .and_then(EditorArtifact::project)
      == Some(file)
  })
}
