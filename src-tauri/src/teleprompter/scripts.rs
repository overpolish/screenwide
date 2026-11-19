// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! Scripts as projects: one plain text file each, listed in the projects
//! window and written and read in the teleprompter.

use std::path::{Path, PathBuf};

use chrono::Local;
use screenwide_project::{Manifest, ScriptManifest};
use serde::Serialize;
use tauri::AppHandle;
use ts_rs::TS;

/// The script's file, in its project's folder: plain text, shown as
/// written. A script made before scripts were plain text keeps the file its
/// manifest names.
const SCRIPT_FILE: &str = "script.txt";

/// A script project's script.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Script {
  pub text: String,
  /// The project's name.
  pub title: String,
}

/// The file of the script project whose manifest is `project`.
fn script_file(project: &Path) -> Result<PathBuf, String> {
  let script = screenwide_project::read(project)
    .map_err(|error| error.to_string())?
    .script
    .ok_or_else(|| "This project holds no script".to_owned())?;
  screenwide_project::resolve(project, &script.file)
}

/// Makes an empty script in the projects folder, remembered as the newest
/// project, and opens the teleprompter on it. Nothing is made when the
/// teleprompter may not be used.
#[tauri::command]
pub fn new_script(app: AppHandle) -> Result<(), String> {
  if !crate::licensing::allow(&app, crate::licensing::Tool::Teleprompter) {
    return Ok(());
  }
  let title = Local::now()
    .naive_local()
    .format("Script %Y-%m-%d at %H.%M.%S")
    .to_string();
  let project = crate::projects_folder::create(&app, &title)?;
  std::fs::write(project.root.join(SCRIPT_FILE), "").map_err(|error| error.to_string())?;
  screenwide_project::write(
    &project.file,
    &Manifest::script(ScriptManifest {
      file: SCRIPT_FILE.to_owned(),
    }),
  )
  .map_err(|error| error.to_string())?;
  screenwide_project::library::remember(&app, &project.file);
  super::window::open(&app, &project.file)
}

#[tauri::command]
pub fn read_script(project: PathBuf) -> Result<Script, String> {
  let text = std::fs::read_to_string(script_file(&project)?).map_err(|error| error.to_string())?;
  Ok(Script {
    text,
    title: screenwide_project::summarize(&project).title,
  })
}

/// Saves `text` as the script of `project`, whole or not at all: written
/// beside the file, then renamed over it.
#[tauri::command]
pub fn save_script(project: PathBuf, text: String) -> Result<(), String> {
  let file = script_file(&project)?;
  let mut writing = file.clone().into_os_string();
  writing.push(".part");
  std::fs::write(&writing, &text).map_err(|error| error.to_string())?;
  std::fs::rename(writing, &file).map_err(|error| error.to_string())?;
  // The manifest is touched so the projects window lists the script as
  // edited now, and its card reads its opening again.
  screenwide_project::update(&project, |_| true)
}
