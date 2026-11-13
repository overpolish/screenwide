// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The library of pictures as the Library island uses it: every picture
//! given to an image, listed, added to from files, dragged out and taken
//! out. Any window showing it reads it again when it changes.

use std::path::PathBuf;

use screenwide_annotations::image::ImageArt;
use screenwide_images::store::{self, Picture};
use tauri::{AppHandle, Emitter};

use super::import::PICTURE_EXTENSIONS;

/// The event every window hears when the library of pictures changes.
const LIBRARY_EVENT: &str = "editor://picture-library";

/// Tells every window the library of pictures has changed: a picture was
/// added, by any way an image takes one, or removed.
pub(crate) fn changed(app: &AppHandle) {
  let _ = app.emit(LIBRARY_EVENT, ());
}

/// Every picture in the library, the most recently added first.
#[tauri::command]
pub async fn get_picture_library() -> Result<Vec<Picture>, String> {
  tauri::async_runtime::spawn_blocking(store::pictures)
    .await
    .map_err(|error| error.to_string())
}

/// Asks for pictures, in a picker parented to the asking window, and adds
/// each to the library, without placing any. Fails only where none of those
/// chosen could be added.
#[tauri::command]
pub async fn add_library_pictures(
  app: AppHandle,
  window: tauri::WebviewWindow,
) -> Result<(), String> {
  let added = app.clone();
  let result = tauri::async_runtime::spawn_blocking(move || {
    use tauri_plugin_dialog::DialogExt;

    let picked: Vec<PathBuf> = app
      .dialog()
      .file()
      .set_title(screenwide_i18n::t!("dialog-choose-pictures"))
      .add_filter(
        screenwide_i18n::t!("dialog-pictures-filter"),
        PICTURE_EXTENSIONS,
      )
      .set_parent(&window)
      .blocking_pick_files()
      .unwrap_or_default()
      .into_iter()
      .filter_map(|path| path.into_path().ok())
      .collect();
    let mut failure = None;
    let mut kept = 0;
    for path in &picked {
      match store::import_file(path) {
        Ok(_) => kept += 1,
        Err(error) => {
          eprintln!("Could not add {} to the library: {error}", path.display());
          failure.get_or_insert(error);
        }
      }
    }
    match failure {
      Some(error) if kept == 0 => Err(error),
      _ => Ok(()),
    }
  })
  .await
  .map_err(|error| error.to_string())?;
  changed(&added);
  result
}

/// Takes the picture `name` out of the library. Projects that show it keep
/// their own copy.
#[tauri::command]
pub async fn remove_library_picture(app: AppHandle, name: String) -> Result<(), String> {
  tauri::async_runtime::spawn_blocking(move || store::remove(&name))
    .await
    .map_err(|error| error.to_string())??;
  changed(&app);
  Ok(())
}

/// Gives the picture `name` the name `title` in the library.
#[tauri::command]
pub async fn rename_library_picture(
  app: AppHandle,
  name: String,
  title: String,
) -> Result<(), String> {
  tauri::async_runtime::spawn_blocking(move || store::rename(&name, &title))
    .await
    .map_err(|error| error.to_string())??;
  changed(&app);
  Ok(())
}

/// The longest side a picture dragged out of the library is drawn at under
/// the pointer, in points.
const DRAGGED_POINTS: f64 = 96.0;

/// Starts dragging the picture `name`'s file out of the calling window, from
/// the press its page is handling now, drawn as the picture itself, so it
/// lands on the picture or the timeline as a file from the Finder does.
#[tauri::command]
pub fn drag_library_picture(window: tauri::WebviewWindow, name: String) -> Result<(), String> {
  let path = store::stored_file(&name)
    .filter(|_| store::is_name(&name))
    .ok_or_else(|| "That picture is not in the library".to_owned())?;
  let (width, height) = image::image_dimensions(&path).unwrap_or((1, 1));
  let scale = DRAGGED_POINTS / f64::from(width.max(height).max(1));
  crate::editor::save::drag_out(
    &window,
    &path,
    &crate::editor::save::DragLook::File {
      height: f64::from(height) * scale,
      width: f64::from(width) * scale,
    },
  )
}

/// Keeps the picture at `path`, dragged in from the system or the library,
/// and says what an image showing it carries.
#[tauri::command]
pub async fn import_library_picture(app: AppHandle, path: PathBuf) -> Result<ImageArt, String> {
  let art = tauri::async_runtime::spawn_blocking(move || store::import_file(&path))
    .await
    .map_err(|error| error.to_string())??;
  changed(&app);
  Ok(art)
}
