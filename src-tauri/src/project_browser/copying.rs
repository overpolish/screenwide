// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Moving projects to another location and duplicating them: work that can
//! copy gigabytes, so it runs off the window's thread and says how far it
//! has got.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use ts_rs::TS;

use crate::project::library;

/// Sent while projects are copied, so the browser can show how far along.
pub(crate) const COPY_PROGRESS_EVENT: &str = "projects://copy-progress";
/// Often enough to move a bar smoothly, rarely enough not to flood the window.
const PROGRESS_INTERVAL: Duration = Duration::from_millis(100);

/// How far a move or duplicate has got, in bytes.
#[derive(Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub(crate) struct CopyProgress {
  copied_bytes: u64,
  total_bytes: u64,
}

/// Runs `each` over `files` off the window's thread, reporting the bytes it
/// copies against everything the projects hold. Every project is tried; the
/// failures are reported together at the end.
async fn copy_each(
  app: AppHandle,
  files: Vec<PathBuf>,
  each: impl Fn(&AppHandle, &PathBuf, u64, &mut dyn FnMut(u64)) -> Result<(), String> + Send + 'static,
) -> Result<(), String> {
  let failures = tauri::async_runtime::spawn_blocking(move || {
    let sizes: Vec<u64> = files
      .iter()
      .map(|file| file.parent().map_or(0, crate::project::folder_size))
      .collect();
    let total_bytes = sizes.iter().sum();
    let mut copied_bytes = 0_u64;
    let mut reported = Instant::now();
    let mut report = |app: &AppHandle, copied_bytes: u64, force: bool| {
      if force || reported.elapsed() >= PROGRESS_INTERVAL {
        reported = Instant::now();
        let _ = app.emit(
          COPY_PROGRESS_EVENT,
          CopyProgress {
            copied_bytes,
            total_bytes,
          },
        );
      }
    };
    report(&app, 0, true);
    let mut failures = Vec::new();
    for (file, size) in files.iter().zip(sizes) {
      let before = copied_bytes;
      let result = each(&app, file, size, &mut |bytes| {
        copied_bytes += bytes;
        report(&app, copied_bytes, false);
      });
      // A failure counts as done, so the bar still reaches its end.
      copied_bytes = before + size;
      report(&app, copied_bytes, true);
      if let Err(error) = result {
        failures.push(error);
      }
    }
    library::notify(&app);
    failures
  })
  .await
  .map_err(|error| error.to_string())?;
  if failures.is_empty() {
    Ok(())
  } else {
    Err(failures.join("\n"))
  }
}

/// Moves projects into `location`. An open one is closed first, as closing
/// its window would, so a name given while open settles before it goes.
/// Recent follows them.
#[tauri::command]
pub async fn move_projects(
  app: AppHandle,
  files: Vec<PathBuf>,
  location: PathBuf,
) -> Result<(), String> {
  crate::project::ensure_default_directory(&app);
  if !location.is_dir() {
    return Err(format!("{} is not available", location.display()));
  }
  let files = files
    .into_iter()
    .map(|file| match crate::editor::open_kind(&app, &file) {
      Some(kind) => crate::editor::close(&app, kind).unwrap_or(file),
      None => file,
    })
    .collect();
  copy_each(app, files, move |app, file, size, on_copied| {
    let moved = crate::project::move_into(file, &location, size, on_copied)?;
    if moved != *file {
      library::moved(app, file, &moved)?;
    }
    Ok(())
  })
  .await
}

/// Copies each project beside itself. An open one is copied as last saved.
#[tauri::command]
pub async fn duplicate_projects(app: AppHandle, files: Vec<PathBuf>) -> Result<(), String> {
  copy_each(app, files, |_, file, _, on_copied| {
    let title = crate::project::summarize(file).title;
    crate::project::duplicate(file, &title, on_copied).map(|_| ())
  })
  .await
}
