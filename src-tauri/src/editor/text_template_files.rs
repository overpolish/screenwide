// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Templates in and out of the app: one imported from a zip or a
//! template's folder into yours, any exported as a zip to share, and one of
//! yours removed. Each change to yours is told to every window, which asks
//! for the templates again. A failure says why in an alert, so the panel
//! has nothing to report itself. And the fonts yours bring, for the
//! webviews to name families in.

use std::path::{Path, PathBuf};

use screenwide_text::{
  remove_user_template, TemplatePackage, TextTemplate, TextTemplateId, MANIFEST_FILE,
};
use serde::Serialize;
use tauri::{AppHandle, Emitter, WebviewWindow};
use tauri_plugin_dialog::DialogExt;
use ts_rs::TS;

/// Sent to every window when your templates change.
const TEXT_TEMPLATES_EVENT: &str = "editor://text-templates";
/// The extension a template's zip carries.
const ZIP: &str = "zip";

/// A font one of your templates brings: the family it names, the weights
/// it covers on the CSS scale, whether it is the italic, and where its file
/// is, which the asset protocol serves.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TextTemplateFont {
  pub family: String,
  pub weights: Vec<u16>,
  pub italic: bool,
  pub path: PathBuf,
}

/// The fonts your templates bring, so a webview names their families in
/// their own type. The app's own templates' are in its style sheet.
#[tauri::command]
pub fn text_template_fonts() -> Vec<TextTemplateFont> {
  screenwide_text::user_template_fonts()
    .into_iter()
    .map(|(font, path)| TextTemplateFont {
      family: font.family,
      weights: font.weights,
      italic: font.italic,
      path,
    })
    .collect()
}

/// Your folder of templates, made if it is not there yet.
fn user_folder(app: &AppHandle) -> Result<PathBuf, String> {
  let folder = super::text_templates::user_templates_folder(app)
    .ok_or_else(|| "The app's data folder cannot be found".to_owned())?;
  std::fs::create_dir_all(&folder)
    .map_err(|error| format!("{} cannot be made: {error}", folder.display()))?;
  Ok(folder)
}

/// The package at `path`: a zip, or the folder a picked manifest is in.
fn read_package(path: &Path) -> Result<TemplatePackage, String> {
  let picked_manifest = path
    .file_name()
    .is_some_and(|name| name.eq_ignore_ascii_case(MANIFEST_FILE));
  match path.parent().filter(|_| picked_manifest) {
    Some(folder) => TemplatePackage::read_folder(folder),
    None => TemplatePackage::read_zip(path),
  }
}

fn import(app: &AppHandle, path: &Path) -> Result<TextTemplate, String> {
  let package = read_package(path)?;
  package.install(&user_folder(app)?)?;
  super::text_templates::hand_over_fonts();
  Ok(package.template)
}

/// Asks for a template's zip, or its folder's manifest, and makes it one of
/// yours, in place of any of yours with its id. Gives the template, or
/// nothing where none was picked or it could not be imported.
#[tauri::command]
pub async fn import_text_template(
  app: AppHandle,
  window: WebviewWindow,
) -> Result<Option<TextTemplate>, String> {
  tauri::async_runtime::spawn_blocking(move || {
    // Parented to the asking window, so the sheet does not attach to some
    // hidden window instead.
    let picked = app
      .dialog()
      .file()
      .set_title(screenwide_i18n::t!("dialog-import-template"))
      .add_filter(
        screenwide_i18n::t!("dialog-template-filter"),
        &[ZIP, "json"],
      )
      .set_parent(&window)
      .blocking_pick_file()
      .and_then(|path| path.into_path().ok())?;
    match import(&app, &picked) {
      Ok(template) => {
        let _ = app.emit(TEXT_TEMPLATES_EVENT, ());
        Some(template)
      }
      Err(error) => {
        crate::alert::show(
          &app,
          &screenwide_i18n::t!("alert-template-import-failed"),
          &error,
        );
        None
      }
    }
  })
  .await
  .map_err(|error| error.to_string())
}

/// Asks where to save template `id` as a zip, and saves it there: one of
/// the app's own as it ships, or one of yours as its folder holds it.
#[tauri::command]
pub async fn export_text_template(
  app: AppHandle,
  window: WebviewWindow,
  id: TextTemplateId,
) -> Result<(), String> {
  tauri::async_runtime::spawn_blocking(move || {
    let package = match TemplatePackage::of(&id) {
      Ok(package) => package,
      Err(error) => {
        crate::alert::show(
          &app,
          &screenwide_i18n::t!("alert-template-export-failed"),
          &error,
        );
        return;
      }
    };
    let Some(destination) = app
      .dialog()
      .file()
      .set_title(screenwide_i18n::t!("dialog-export-template"))
      .set_file_name(format!("{id}.{ZIP}"))
      .add_filter(screenwide_i18n::t!("dialog-template-filter"), &[ZIP])
      .set_parent(&window)
      .blocking_save_file()
      .and_then(|path| path.into_path().ok())
    else {
      return;
    };
    if let Err(error) = package.write_zip(&destination) {
      crate::alert::show(
        &app,
        &screenwide_i18n::t!("alert-template-export-failed"),
        &error,
      );
    }
  })
  .await
  .map_err(|error| error.to_string())
}

/// Moves your template `id` to the Trash, or the Recycle Bin. Captions in
/// it keep their style, as they do in any template the app does not have.
#[tauri::command]
pub async fn remove_text_template(app: AppHandle, id: TextTemplateId) -> Result<(), String> {
  tauri::async_runtime::spawn_blocking(move || {
    match user_folder(&app).and_then(|folder| remove_user_template(&id, &folder)) {
      Ok(()) => {
        let _ = app.emit(TEXT_TEMPLATES_EVENT, ());
      }
      Err(error) => crate::alert::show(
        &app,
        &screenwide_i18n::t!("alert-template-remove-failed"),
        &error,
      ),
    }
  })
  .await
  .map_err(|error| error.to_string())
}
